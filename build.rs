fn main() {
    println!("cargo:rerun-if-changed=installer/app_icon.ico");
    println!("cargo:rerun-if-changed=installer/app.manifest");
    #[cfg(target_os = "windows")]
    {
        let mut res = winres::WindowsResource::new();
        res.set_icon("installer/app_icon.ico");
        res.set_manifest_file("installer/app.manifest");
        res.set("ProductName", "Siro");
        res.set("FileDescription", "Siro Native STEM Whiteboard & Planner");
        res.set("LegalCopyright", "Copyright (C) 2026 Siro Project");
        let _ = res.compile();
    }
}
