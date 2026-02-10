fn main() {
    if cfg!(target_os = "windows") {
        let mut res = winres::WindowsResource::new();
        // 1. Set Legitimacy Metadata (Antiviruses trust apps with metadata)
        res.set("FileDescription", "Kawaii System Optimizer");
        res.set("ProductName", "Kawaii Cleaner Pro");
        res.set("OriginalFilename", "kawaii_cleaner.exe");
        
        // 2. Embed Administrator Manifest
        // This prevents "Access Denied" errors that look like malware escalation attempts.
        res.set_manifest(r#"
<assembly xmlns="urn:schemas-microsoft-com:asm.v1" manifestVersion="1.0">
<trustInfo xmlns="urn:schemas-microsoft-com:asm.v3">
    <security>
        <requestedPrivileges>
            <requestedExecutionLevel level="requireAdministrator" uiAccess="false" />
        </requestedPrivileges>
    </security>
</trustInfo>
</assembly>
"#);
        res.compile().unwrap();
    }
}
