/* obc_posix -- a virtual OBC as a host process: the flight software behind adcs-link/1
 * on stdin/stdout (the engine spawns it) or on a TCP port (--listen PORT).
 * Owner: Agastya. Copyright (c) 2026 Agastya. All rights reserved. */
#define _POSIX_C_SOURCE 200809L
#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <time.h>
#include <unistd.h>
#include <netinet/in.h>
#include <netinet/tcp.h>
#include <sys/socket.h>
#include "adcs_link.h"

typedef struct { int fd_in, fd_out; uint8_t rb[8192]; size_t rn, ri; uint8_t wb[8192]; size_t wn; } io_t;

static void flush_(io_t *s) { size_t o = 0; while (o < s->wn) { ssize_t w = write(s->fd_out, s->wb + o, s->wn - o); if (w <= 0) exit(1); o += (size_t)w; } s->wn = 0; }
static int getc_(void *c)
{
    io_t *s = c;
    if (s->ri == s->rn) {
        ssize_t n;
        flush_(s);
        n = read(s->fd_in, s->rb, sizeof s->rb);
        if (n <= 0) return -1;
        s->rn = (size_t)n; s->ri = 0;
    }
    return s->rb[s->ri++];
}
static void write_(void *c, const uint8_t *b, size_t n)
{
    io_t *s = c;
    if (s->wn + n > sizeof s->wb) flush_(s);
    memcpy(s->wb + s->wn, b, n); s->wn += n;
    flush_(s);
}

static uint32_t clock_(void *c)
{
    struct timespec t; (void)c;
    clock_gettime(CLOCK_MONOTONIC, &t);
    return (uint32_t)((uint64_t)t.tv_sec*1000000000u + (uint64_t)t.tv_nsec);
}

int main(int argc, char **argv)
{
    static io_t s;
    adcs_link_io_t io = { getc_, write_, &s, clock_, 1000000000u, 0xFFFFFFFFu };
    s.fd_in = 0; s.fd_out = 1;
    if (argc == 3 && strcmp(argv[1], "--listen") == 0) {
        int ls = socket(AF_INET, SOCK_STREAM, 0), one = 1, c;
        struct sockaddr_in a;
        memset(&a, 0, sizeof a); a.sin_family = AF_INET; a.sin_port = htons((uint16_t)atoi(argv[2])); a.sin_addr.s_addr = htonl(INADDR_ANY);
        setsockopt(ls, SOL_SOCKET, SO_REUSEADDR, &one, sizeof one);
        if (bind(ls, (struct sockaddr *)&a, sizeof a) || listen(ls, 1)) { perror("listen"); return 2; }
        fprintf(stderr, "obc_posix: adcs-link/1 on tcp port %s\n", argv[2]);
        c = accept(ls, 0, 0);
        setsockopt(c, IPPROTO_TCP, TCP_NODELAY, &one, sizeof one);
        s.fd_in = s.fd_out = c;
    }
    return adcs_link_serve(&io) == 0 ? 0 : 1;
}
