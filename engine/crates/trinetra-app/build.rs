//! On Windows, the program carries its icon and its version information (company, product,
//! version, description), so it is a named program with a face rather than an unknown file.
//! Other systems need nothing here: the macOS bundle is made by tools/macapp.py.
//! Owner: Agastya. Copyright (c) 2026 Agastya. All rights reserved.
fn main() {
    println!("cargo:rerun-if-changed=icon/trinetra.ico");
    println!("cargo:rerun-if-changed=../../../VERSION");
    if std::env::var("CARGO_CFG_TARGET_OS").as_deref() != Ok("windows") { return; }
    let v = std::fs::read_to_string("../../../VERSION").unwrap_or_else(|_| "1.0.0".into());
    let v = v.trim();
    let mut n: Vec<u16> = v.split('.').map(|x| x.parse().unwrap_or(0)).collect();
    n.resize(4, 0);
    let out = std::path::PathBuf::from(std::env::var("OUT_DIR").unwrap());
    let ico = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("icon/trinetra.ico");
    let rc = format!(r#"#include <winver.h>
1 ICON "{ico}"
1 VERSIONINFO
FILEVERSION {a},{b},{c},{d}
PRODUCTVERSION {a},{b},{c},{d}
FILEOS VOS_NT_WINDOWS32
FILETYPE VFT_APP
BEGIN
  BLOCK "StringFileInfo"
  BEGIN
    BLOCK "040904b0"
    BEGIN
      VALUE "CompanyName", "Agastya"
      VALUE "FileDescription", "TRI-NETRA ADCS"
      VALUE "FileVersion", "{v}"
      VALUE "InternalName", "trinetra-app"
      VALUE "LegalCopyright", "Copyright (c) 2026 Agastya. All rights reserved."
      VALUE "OriginalFilename", "TRI-NETRA ADCS.exe"
      VALUE "ProductName", "TRI-NETRA ADCS"
      VALUE "ProductVersion", "{v}"
    END
  END
  BLOCK "VarFileInfo"
  BEGIN
    VALUE "Translation", 0x409, 1200
  END
END
"#, ico = ico.display().to_string().replace('\\', "\\\\"), a = n[0], b = n[1], c = n[2], d = n[3]);
    let f = out.join("trinetra.rc");
    std::fs::write(&f, rc).unwrap();
    embed_resource::compile(&f, embed_resource::NONE).manifest_optional().unwrap();
}
