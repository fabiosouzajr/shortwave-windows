; Shortwave - Windows installer (Inno Setup 6)
; Copyright (C) 2026  The Shortwave Windows port contributors
;
; This program is free software: you can redistribute it and/or modify
; it under the terms of the GNU General Public License as published by
; the Free Software Foundation, either version 3 of the License, or
; (at your option) any later version.
;
; Build with build-aux\windows\installer.ps1, or by hand:
;   ISCC.exe /DAppVersion=5.0.0-win.1 packaging\windows\shortwave.iss
; Expects the bundle in dist\Shortwave (build-aux\windows\bundle.ps1).

#ifndef AppVersion
  #define AppVersion "5.0.0-win.1"
#endif
#define AppName "Shortwave"
#define AppId "de.haeckerfelix.Shortwave"
#define AppExe "bin\shortwave.exe"
#define Root "..\.."

[Setup]
; Never change AppId: upgrades and uninstall depend on it
AppId={{8CB9C583-1BED-4EE5-B3C7-585B250292C4}
AppName={#AppName}
AppVersion={#AppVersion}
AppVerName={#AppName} {#AppVersion}
AppPublisher=Unofficial Windows port of GNOME Shortwave
AppPublisherURL=https://gitlab.gnome.org/World/Shortwave
AppSupportURL=https://gitlab.gnome.org/World/Shortwave
AppCopyright=Felix Häcker and contributors (GPL-3.0-or-later)
DefaultDirName={autopf}\{#AppName}
DefaultGroupName={#AppName}
DisableProgramGroupPage=yes
LicenseFile={#Root}\COPYING.md
InfoBeforeFile={#Root}\NOTICE.md
OutputDir={#Root}\dist
OutputBaseFilename=Shortwave-{#AppVersion}-windows-x64-setup
SetupIconFile={#Root}\build-aux\windows\shortwave.ico
UninstallDisplayIcon={app}\{#AppExe}
UninstallDisplayName={#AppName}
ArchitecturesAllowed=x64compatible
ArchitecturesInstallIn64BitMode=x64compatible
MinVersion=10.0
; Per-machine by default (needed for the firewall rule); per-user allowed
PrivilegesRequired=admin
PrivilegesRequiredOverridesAllowed=dialog
Compression=lzma2/max
SolidCompression=yes
WizardStyle=modern
CloseApplications=yes
RestartApplications=no

[Languages]
Name: "en"; MessagesFile: "compiler:Default.isl"
Name: "ptbr"; MessagesFile: "compiler:Languages\BrazilianPortuguese.isl"

[CustomMessages]
en.FirewallRule=Allow discovering Google Cast devices on the local network (Windows Firewall rule for mDNS, UDP 5353)
ptbr.FirewallRule=Permitir encontrar dispositivos Google Cast na rede local (regra do Firewall do Windows para mDNS, UDP 5353)

[Tasks]
Name: "desktopicon"; Description: "{cm:CreateDesktopIcon}"; GroupDescription: "{cm:AdditionalIcons}"; Flags: unchecked
Name: "firewall"; Description: "{cm:FirewallRule}"; Check: IsAdminInstallMode

[Files]
Source: "{#Root}\dist\Shortwave\*"; DestDir: "{app}"; Flags: ignoreversion recursesubdirs createallsubdirs

[Icons]
; The AppUserModelID ties toast notifications and the taskbar to this shortcut
Name: "{autoprograms}\{#AppName}"; Filename: "{app}\{#AppExe}"; AppUserModelID: "{#AppId}"; Comment: "Listen to internet radio"
Name: "{autodesktop}\{#AppName}"; Filename: "{app}\{#AppExe}"; AppUserModelID: "{#AppId}"; Tasks: desktopicon

[Run]
Filename: "{sys}\netsh.exe"; \
  Parameters: "advfirewall firewall add rule name=""Shortwave (mDNS)"" dir=in action=allow protocol=UDP localport=5353 program=""{app}\{#AppExe}"" enable=yes"; \
  Flags: runhidden; Tasks: firewall; StatusMsg: "Windows Firewall"
Filename: "{app}\{#AppExe}"; Description: "{cm:LaunchProgram,{#AppName}}"; Flags: nowait postinstall skipifsilent

[UninstallRun]
Filename: "{sys}\netsh.exe"; \
  Parameters: "advfirewall firewall delete rule name=""Shortwave (mDNS)"" program=""{app}\{#AppExe}"""; \
  Flags: runhidden; RunOnceId: "RemoveFirewallRule"; Check: IsAdminInstallMode

[UninstallDelete]
; GStreamer's plugin registry cache, if one was written next to the app
Type: filesandordirs; Name: "{app}\lib\gstreamer-1.0\registry.*"
