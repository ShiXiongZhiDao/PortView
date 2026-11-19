fn main() {
    // 匿名使用统计（ADR-0003）：CI 用 Secret 注入，本机从同目录 .env 读。
    // 两者都没有就不注入：lib.rs 的 option_env! 得到 None → 传空 key，插件静默禁用。
    // 任何人 clone 下来都能编译，默认不发数据；key 不进 git，也不进公开树。
    if std::env::var_os("PORTVIEW_APTABASE_KEY").is_none() {
        if let Ok(contents) = std::fs::read_to_string(".env") {
            for line in contents.lines() {
                if let Some((key, value)) = line.trim().split_once('=') {
                    if key.trim() == "PORTVIEW_APTABASE_KEY" {
                        let value = value.trim().trim_matches(['"', '\'']);
                        if !value.is_empty() {
                            println!("cargo:rustc-env=PORTVIEW_APTABASE_KEY={value}");
                        }
                        break;
                    }
                }
            }
        }
    }

    // 以管理员权限运行（UAC 提权），保证能读取所有进程完整路径并结束系统进程
    let mut windows = tauri_build::WindowsAttributes::new();
    windows = windows.app_manifest(
        r#"
<assembly xmlns="urn:schemas-microsoft-com:asm.v1" manifestVersion="1.0">
  <dependency>
    <dependentAssembly>
      <assemblyIdentity
        type="win32"
        name="Microsoft.Windows.Common-Controls"
        version="6.0.0.0"
        processorArchitecture="*"
        publicKeyToken="6595b64144ccf1df"
        language="*"
      />
    </dependentAssembly>
  </dependency>
  <trustInfo xmlns="urn:schemas-microsoft-com:asm.v3">
    <security>
      <requestedPrivileges>
        <requestedExecutionLevel level="requireAdministrator" uiAccess="false" />
      </requestedPrivileges>
    </security>
  </trustInfo>
</assembly>
"#,
    );
    let attrs = tauri_build::Attributes::new().windows_attributes(windows);
    tauri_build::try_build(attrs).expect("failed to run tauri build script");
}
