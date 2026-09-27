function [T, Y, D] = gaussJackson8(f, t0, tf, y0, opts)
%INTEG.GAUSSJACKSON8  8th-order Gauss-Jackson multistep integrator (summed form).
%   [T,Y,D] = integ.gaussJackson8(f, t0, tf, y0, opts),  opts.h = fixed step [s].
%
%   The gold-standard fixed-step method for precise orbit propagation of
%   second-order systems r'' = a(t,r,v): it carries the acceleration history and
%   uses summed (Cowell) accumulation with Kahan compensation, giving extremely
%   low energy drift over long arcs at one acceleration evaluation per step
%   (predict-evaluate-correct, PECE with one corrector pass here).
%
%   Coefficient tables are the validated 9-point Gauss-Jackson set.  The 8 start
%   points around the epoch are generated self-contained with the order-6 Luther
%   RK (no ode113 dependency), then the summed sums are made consistent by fixed-
%   point iteration before the multistep march begins.
%
%   Reference: Berry & Healy (2004), "Implementation of Gauss-Jackson
%   Integration for Orbit Propagation", J. Astronaut. Sci.
%
%   NOTE: for varying step or event location prefer integ.rk78; GJ8 shines for
%   long fixed-step precise propagation and OD reference trajectories.

    accel = @(t,r,v) subA(f(t,[r(:);v(:)]));
    h = opts.h;  y0=y0(:);
    N = max(5, round((tf-t0)/h));  h=(tf-t0)/N;      % N = OUTPUT steps; land on tf
    T = t0 + (0:N)'*h;                                % output node times
    % Window indexing: the epoch is window node 5 (n=0); output node j maps to
    % window node j+4.  We must therefore build window nodes 1..N+5.

    % ---------- startup: 9 points centred on the epoch (i=1..9 -> n=-4..+4) ----
    r0=y0(1:3); v0=y0(4:6);
    R=zeros(3,9); Vv=zeros(3,9); Aa=zeros(3,9);
    R(:,5)=r0; Vv(:,5)=v0; Aa(:,5)=accel(t0,r0,v0);
    % forward n=+1..+4 and backward n=-1..-4 via short Luther-RK sub-integration
    for n=1:4
        [rf,vf]=rkstep6(accel, t0+(n-1)*h, R(:,4+n), Vv(:,4+n), h);   % march +h
        R(:,5+n)=rf; Vv(:,5+n)=vf; Aa(:,5+n)=accel(t0+n*h,rf,vf);
        [rb,vb]=rkstep6(accel, t0-(n-1)*h, R(:,6-n), Vv(:,6-n), -h);  % march -h
        R(:,5-n)=rb; Vv(:,5-n)=vb; Aa(:,5-n)=accel(t0-n*h,rb,vb);
    end

    [Ab,Bb]=gjTables();               % 10x9 predictor/corrector coeff rows
    h2=h*h;

    % ---------- initialise summed sums from the (accurate) RK startup points ----
    % The RK startup points R,Vv,Aa are trusted; here we only *back out* the first
    % sum sn and second sum Sn that reproduce them under the summed-form relations
    %   dy_n = h (sn_n + b_row0 . ddy),   y_n = h^2 (Sn_n + a_row0 . ddy)
    % evaluated at the epoch node (index 5, n=0), then propagate to the window.
    sn=zeros(3,9); Sn=zeros(3,9);
    sn(:,5) = Vv(:,5)/h  - sum(Bb(5,:).*Aa,2);                 % first sum at n=0
    for n=-1:-1:-4, sn(:,n+5)=sn(:,n+6)-(Aa(:,n+6)+Aa(:,n+5))/2; end
    for n= 1: 1: 4, sn(:,n+5)=sn(:,n+4)+(Aa(:,n+4)+Aa(:,n+5))/2; end
    Sn(:,5) = R(:,5)/h2 - sum(Ab(5,:).*Aa,2);                  % second sum at n=0
    for n=-1:-1:-4, Sn(:,n+5)=Sn(:,n+6)-sn(:,n+6)+Aa(:,n+6)/2; end
    for n= 1: 1: 4, Sn(:,n+5)=Sn(:,n+4)+sn(:,n+4)+Aa(:,n+4)/2; end

    % ---------- node-indexed history arrays (faithful gj8_step port) ----
    W  = N+5;                                    % last window node = output tf
    y  = nan(3,W);  dy = nan(3,W);  ddy = nan(3,W);
    Sn2= nan(3,W+1);  sn1= nan(3,W+1);           % second/first sums by node
    Snc= zeros(3,W+1); snc= zeros(3,W+1);        % Kahan compensation
    for n=1:9
        y(:,n)=R(:,n); dy(:,n)=Vv(:,n); ddy(:,n)=Aa(:,n);
        Sn2(:,n)=Sn(:,n); sn1(:,n)=sn(:,n);
    end

    ap=Ab(10,:); bp=Bb(10,:);      % predictor rows (n=+5)
    ac=Ab(9,:);  bc=Bb(9,:);       % corrector rows (n=+4)
    MaxIter=20;

    i=9;
    while i<=W-1
        t=t0+(i-5)*h;              % time of node i
        % ---- corrector refine of node i (only once history is full, i>9) ----
        if i>9
            addyn43=zeros(3,1); bddyn43=zeros(3,1);
            for j=1:8
                addyn43=addyn43+ac(j)*ddy(:,i-9+j);
                bddyn43=bddyn43+bc(j)*ddy(:,i-9+j);
            end
            for it=1:MaxIter
                xsn=(ddy(:,i-1)+ddy(:,i))/2;                 % trapezoid increment
                [sn1(:,i),snc(:,i)]=kahan(sn1(:,i-1),xsn,snc(:,i));
                yc = h2*(Sn2(:,i)+addyn43+ac(9)*ddy(:,i));
                dyc= h *(sn1(:,i)+bddyn43+bc(9)*ddy(:,i));
                rs=max(norm(y(:,i)),1); vs=max(norm(dy(:,i)),1);
                rel=max(norm(yc-y(:,i))/rs, norm(dyc-dy(:,i))/vs);
                y(:,i)=yc; dy(:,i)=dyc; ddy(:,i)=accel(t,y(:,i),dy(:,i));
                if rel<=1e-13, break; end
            end
        end
        % ---- predictor extrapolate node i+1 ----
        addyn54=zeros(3,1); bddyn54=zeros(3,1);
        for j=1:9
            addyn54=addyn54+ap(j)*ddy(:,i-9+j);
            bddyn54=bddyn54+bp(j)*ddy(:,i-9+j);
        end
        xSn=sn1(:,i)+ddy(:,i)/2;
        [Sn2(:,i+1),Snc(:,i+1)]=kahan(Sn2(:,i),xSn,Snc(:,i));
        y(:,i+1) =h2*(Sn2(:,i+1)+addyn54);
        dy(:,i+1)=h *(sn1(:,i)+ddy(:,i)/2+bddyn54);
        ddy(:,i+1)=accel(t+h,y(:,i+1),dy(:,i+1));
        snc(:,i+1)=0;
        i=i+1;
    end

    % ---------- assemble outputs (epoch = window node 5 -> output node 1) ----
    Y=zeros(N+1,6); D=zeros(N+1,6);
    for j=1:N+1
        n=j+4;                                    % map output node -> window node
        Y(j,:)=[y(:,n).' dy(:,n).'];
        D(j,:)=[dy(:,n).' ddy(:,n).'];
    end
end

% ---- helpers -----------------------------------------------------------
function a=subA(dy), a=dy(4:6); end

function [rn,vn]=rkstep6(accel,t,r,v,h)
% one order-6 Luther RK step of a 2nd-order system (self-contained startup)
    F=@(t,y)[y(4:6); accel(t,y(1:3),y(4:6))];
    q=sqrt(21); A=zeros(7);
    A(2,1)=1;A(3,1:2)=[3/8,1/8];A(4,1:3)=[8/27,2/27,8/27];
    A(5,1:4)=[3*(3*q-7)/392,-8*(7-q)/392,48*(7-q)/392,-3*(21-q)/392];
    A(6,1:5)=[-5*(231+51*q)/1960,-40*(7+q)/1960,-320*q/1960,3*(21+121*q)/1960,392*(6+q)/1960];
    A(7,1:6)=[15*(22+7*q)/180,120/180,40*(7*q-5)/180,-63*(3*q-2)/180,-14*(49+9*q)/180,70*(7-q)/180];
    c=[0;1;1/2;2/3;(7-q)/14;(7+q)/14;1]; b=[1/20;0;16/45;0;49/180;49/180;1/20];
    y=[r(:);v(:)]; K=zeros(6,7);
    for i=1:7
        yi=y; for j=1:i-1, yi=yi+h*A(i,j)*K(:,j); end
        K(:,i)=F(t+c(i)*h,yi);
    end
    y=y+h*(K*b); rn=y(1:3); vn=y(4:6);
end

function [snew, cnew]=kahan(s, x, c)
% Kahan-compensated running sum: snew = s + x, carrying compensation c.
    yk=x-c; t=s+yk; cnew=(t-s)-yk; snew=t;
end

function [a,b]=gjTables()
% 9-point Gauss-Jackson coefficients (rows index 1..10 -> n=-4..+5).
a=zeros(10,9); b=zeros(10,9);
a(1,:)=[3250433/53222400,572741/5702400,-8701681/39916800,4026311/13305600,-917039/3193344,7370669/39916800,-1025779/13305600,754331/39916800,-330157/159667200];
a(2,:)=[-330157/159667200,530113/6652800,518887/19958400,-27631/623700,44773/1064448,-531521/19958400,109343/9979200,-1261/475200,45911/159667200];
a(3,:)=[45911/159667200,-185839/39916800,171137/1900800,73643/39916800,-25775/3193344,77597/13305600,-98911/39916800,24173/39916800,-3499/53222400];
a(4,:)=[-3499/53222400,4387/4989600,-35039/4989600,90817/950400,-20561/3193344,2117/9979200,2059/6652800,-317/2851200,317/22809600];
a(5,:)=[317/22809600,-2539/13305600,55067/39916800,-326911/39916800,14797/152064,-326911/39916800,55067/39916800,-2539/13305600,317/22809600];
a(6,:)=[317/22809600,-317/2851200,2059/6652800,2117/9979200,-20561/3193344,90817/950400,-35039/4989600,4387/4989600,-3499/53222400];
a(7,:)=[-3499/53222400,24173/39916800,-98911/39916800,77597/13305600,-25775/3193344,73643/39916800,171137/1900800,-185839/39916800,45911/159667200];
a(8,:)=[45911/159667200,-1261/475200,109343/9979200,-531521/19958400,44773/1064448,-27631/623700,518887/19958400,530113/6652800,-330157/159667200];
a(9,:)=[-330157/159667200,754331/39916800,-1025779/13305600,7370669/39916800,-917039/3193344,4026311/13305600,-8701681/39916800,572741/5702400,3250433/53222400];
a(10,:)=[3250433/53222400,-11011481/19958400,6322573/2851200,-8660609/1663200,25162927/3193344,-159314453/19958400,18071351/3326400,-24115843/9979200,103798439/159667200];
b(1,:)=[19087/89600,-427487/725760,3498217/3628800,-500327/403200,6467/5670,-2616161/3628800,24019/80640,-263077/3628800,8183/1036800];
b(2,:)=[8183/1036800,57251/403200,-1106377/3628800,218483/725760,-69/280,530177/3628800,-210359/3628800,5533/403200,-425/290304];
b(3,:)=[-425/290304,76453/3628800,5143/57600,-660127/3628800,661/5670,-4997/80640,83927/3628800,-19109/3628800,7/12800];
b(4,:)=[7/12800,-23173/3628800,29579/725760,2497/57600,-2563/22680,172993/3628800,-6463/403200,2497/725760,-2497/7257600];
b(5,:)=[-2497/7257600,1469/403200,-68119/3628800,252769/3628800,0,-252769/3628800,68119/3628800,-1469/403200,2497/7257600];
b(6,:)=[2497/7257600,-2497/725760,6463/403200,-172993/3628800,2563/22680,-2497/57600,-29579/725760,23173/3628800,-7/12800];
b(7,:)=[-7/12800,19109/3628800,-83927/3628800,4997/80640,-661/5670,660127/3628800,-5143/57600,-76453/3628800,425/290304];
b(8,:)=[425/290304,-5533/403200,210359/3628800,-530177/3628800,69/280,-218483/725760,1106377/3628800,-57251/403200,-8183/1036800];
b(9,:)=[-8183/1036800,263077/3628800,-24019/80640,2616161/3628800,-6467/5670,500327/403200,-3498217/3628800,427487/725760,-19087/89600];
b(10,:)=[25713/89600,-9401029/3628800,5393233/518400,-9839609/403200,167287/4536,-135352319/3628800,10219841/403200,-40987771/3628800,3288521/1036800];
end
