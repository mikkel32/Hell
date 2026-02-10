fn main() {
    if cfg!(target_os = "windows") {
        let mut res = winres::WindowsResource::new();
        res.set("FileDescription", "Kawaii Organizer Utility");
        res.set("ProductName", "Kawaii Organizer");
        res.set("OriginalFilename", "kawaii_organizer.exe");
        
        // CRITICAL: Request 'asInvoker' (Standard User).
        // This avoids the scary UAC prompt and lowers the Heuristic Risk score.
        res.set_manifest(r#"
<assembly xmlns="urn:schemas-microsoft-com:asm.v1" manifestVersion="1.0">
<trustInfo xmlns="urn:schemas-microsoft-com:asm.v3">
    <security>
        <requestedPrivileges>
            <requestedExecutionLevel level="asInvoker" uiAccess="false" />
        </requestedPrivileges>
    </security>
</trustInfo>
</assembly>
"#);
        res.compile().unwrap();
    }
}
