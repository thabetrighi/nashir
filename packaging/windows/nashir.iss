; Nashir (ناشر) — Windows installer (Inno Setup 6).
;
; Packages the folder `nsh package App.tsx --target windows` writes
; (Nashir.exe + nsh\), with Start menu and optional desktop shortcuts,
; .md / .markdown / .mdown / .mkd file associations and an uninstaller.
;
; Build (from the repository root, after packaging the app):
;   iscc /DAppVersion=0.1.0 /DSourceDir=app\.nsh\out\Nashir /DOutputDir=dist packaging\windows\nashir.iss
;
; Code signing: pass /DSignTool=nashirsign and register the tool with
; iscc's /S switch, e.g.
;   /Snashirsign="signtool sign /fd sha256 /tr http://timestamp.digicert.com /td sha256 /f cert.pfx /p $p $f"
; (the release workflow does this when its signing secrets are set).

#ifndef AppVersion
  #define AppVersion "0.1.0"
#endif
#ifndef SourceDir
  #define SourceDir "..\..\app\.nsh\out\Nashir"
#endif
#ifndef OutputDir
  #define OutputDir "..\..\dist"
#endif

#define AppName "Nashir"
#define AppNameAr "ناشر"
#define AppExe "Nashir.exe"
#define AppId "app.nashir.reader"
#define ProgId "Nashir.Markdown"

[Setup]
AppId={{6E1F4C2A-9B5D-4E8A-A7C3-5D2B1F0E9A41}
AppName={#AppName}
AppVerName={#AppName} {#AppVersion}
AppVersion={#AppVersion}
AppPublisher=Nashir contributors
AppPublisherURL=https://github.com/thabetrighi/nashir
AppSupportURL=https://github.com/thabetrighi/nashir/issues
AppUpdatesURL=https://github.com/thabetrighi/nashir/releases
VersionInfoVersion={#AppVersion}
VersionInfoDescription={#AppName} — Markdown reader
DefaultDirName={autopf}\{#AppName}
DefaultGroupName={#AppName}
DisableProgramGroupPage=yes
; Per-user by default (no admin prompt); users may choose all users.
PrivilegesRequired=lowest
PrivilegesRequiredOverridesAllowed=dialog
ArchitecturesAllowed=x64compatible
ArchitecturesInstallIn64BitMode=x64compatible
; Windows 10 1809 is NSH's minimum.
MinVersion=10.0.17763
OutputDir={#OutputDir}
OutputBaseFilename=Nashir-{#AppVersion}-windows-x64-setup
SetupIconFile=nashir.ico
UninstallDisplayIcon={app}\{#AppExe}
UninstallDisplayName={#AppName} ({#AppNameAr})
WizardStyle=modern
Compression=lzma2/max
SolidCompression=yes
ChangesAssociations=yes
CloseApplications=yes
#ifdef SignTool
SignTool={#SignTool}
SignedUninstaller=yes
#endif

[Languages]
Name: "arabic"; MessagesFile: "compiler:Languages\Arabic.isl"
Name: "english"; MessagesFile: "compiler:Default.isl"
Name: "french"; MessagesFile: "compiler:Languages\French.isl"

[CustomMessages]
arabic.AssocTask=اجعل ناشر يفتح ملفات Markdown (.md و.markdown و.mdown و.mkd)
english.AssocTask=Open Markdown files (.md, .markdown, .mdown, .mkd) with Nashir
french.AssocTask=Ouvrir les fichiers Markdown (.md, .markdown, .mdown, .mkd) avec Nashir
arabic.MarkdownDoc=مستند Markdown
english.MarkdownDoc=Markdown document
french.MarkdownDoc=Document Markdown

[Tasks]
Name: "desktopicon"; Description: "{cm:CreateDesktopIcon}"; GroupDescription: "{cm:AdditionalIcons}"
Name: "associate"; Description: "{cm:AssocTask}"; GroupDescription: "{cm:AssocFileExtension,{#AppName},.md}"

[Files]
Source: "{#SourceDir}\*"; DestDir: "{app}"; Flags: ignoreversion recursesubdirs createallsubdirs

[Icons]
Name: "{autoprograms}\{#AppName}"; Filename: "{app}\{#AppExe}"; AppUserModelID: "{#AppId}"
Name: "{autodesktop}\{#AppName}"; Filename: "{app}\{#AppExe}"; Tasks: desktopicon; AppUserModelID: "{#AppId}"

[Registry]
; The ProgID: what "Open" does, and the icon of Markdown files.
Root: HKA; Subkey: "Software\Classes\{#ProgId}"; ValueType: string; ValueName: ""; ValueData: "{cm:MarkdownDoc}"; Flags: uninsdeletekey
Root: HKA; Subkey: "Software\Classes\{#ProgId}\DefaultIcon"; ValueType: string; ValueName: ""; ValueData: """{app}\{#AppExe}"",0"
Root: HKA; Subkey: "Software\Classes\{#ProgId}\shell\open\command"; ValueType: string; ValueName: ""; ValueData: """{app}\{#AppExe}"" ""%1"""
; Nashir is always offered in "Open with" for these extensions…
Root: HKA; Subkey: "Software\Classes\.md\OpenWithProgids"; ValueType: string; ValueName: "{#ProgId}"; ValueData: ""; Flags: uninsdeletevalue
Root: HKA; Subkey: "Software\Classes\.markdown\OpenWithProgids"; ValueType: string; ValueName: "{#ProgId}"; ValueData: ""; Flags: uninsdeletevalue
Root: HKA; Subkey: "Software\Classes\.mdown\OpenWithProgids"; ValueType: string; ValueName: "{#ProgId}"; ValueData: ""; Flags: uninsdeletevalue
Root: HKA; Subkey: "Software\Classes\.mkd\OpenWithProgids"; ValueType: string; ValueName: "{#ProgId}"; ValueData: ""; Flags: uninsdeletevalue
; …and becomes their default handler when the user ticks the task.
Root: HKA; Subkey: "Software\Classes\.md"; ValueType: string; ValueName: ""; ValueData: "{#ProgId}"; Flags: uninsdeletevalue; Tasks: associate
Root: HKA; Subkey: "Software\Classes\.markdown"; ValueType: string; ValueName: ""; ValueData: "{#ProgId}"; Flags: uninsdeletevalue; Tasks: associate
Root: HKA; Subkey: "Software\Classes\.mdown"; ValueType: string; ValueName: ""; ValueData: "{#ProgId}"; Flags: uninsdeletevalue; Tasks: associate
Root: HKA; Subkey: "Software\Classes\.mkd"; ValueType: string; ValueName: ""; ValueData: "{#ProgId}"; Flags: uninsdeletevalue; Tasks: associate
; Registered application (Default Apps settings, "Open with" names).
Root: HKA; Subkey: "Software\Classes\Applications\{#AppExe}"; ValueType: string; ValueName: "FriendlyAppName"; ValueData: "{#AppName}"; Flags: uninsdeletekey
Root: HKA; Subkey: "Software\Classes\Applications\{#AppExe}\SupportedTypes"; ValueType: string; ValueName: ".md"; ValueData: ""
Root: HKA; Subkey: "Software\Classes\Applications\{#AppExe}\SupportedTypes"; ValueType: string; ValueName: ".markdown"; ValueData: ""
Root: HKA; Subkey: "Software\Classes\Applications\{#AppExe}\SupportedTypes"; ValueType: string; ValueName: ".mdown"; ValueData: ""
Root: HKA; Subkey: "Software\Classes\Applications\{#AppExe}\SupportedTypes"; ValueType: string; ValueName: ".mkd"; ValueData: ""

[Run]
Filename: "{app}\{#AppExe}"; Description: "{cm:LaunchProgram,{#AppName}}"; Flags: nowait postinstall skipifsilent

[UninstallDelete]
; Settings live in %APPDATA%\app.nashir.reader; they are the user's and are
; kept on uninstall (documented in docs/settings-and-storage.md).
