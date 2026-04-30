; NSIS installer template for SecureImage Forge desktop.
; Build with: makensis -DVERSION=0.1.0 forge-desktop.nsi
;
; Authenticode signing happens after `makensis` produces the .exe — see
; packaging/windows/sign.ps1.

!ifndef VERSION
  !define VERSION "0.0.0"
!endif

!define APPNAME "SecureImage Forge"
!define COMPANY "SecureImage"
!define INSTALL_DIR "$PROGRAMFILES64\SecureImage\Forge"

OutFile "SecureImageForge-${VERSION}.exe"
InstallDir "${INSTALL_DIR}"
RequestExecutionLevel admin
SetCompressor /SOLID lzma

VIProductVersion "${VERSION}.0"
VIAddVersionKey "ProductName"     "${APPNAME}"
VIAddVersionKey "CompanyName"     "${COMPANY}"
VIAddVersionKey "FileDescription" "${APPNAME}"
VIAddVersionKey "FileVersion"     "${VERSION}"
VIAddVersionKey "ProductVersion"  "${VERSION}"
VIAddVersionKey "LegalCopyright"  "Apache-2.0"

Page directory
Page instfiles

Section "Install"
    SetOutPath "$INSTDIR"
    File "/oname=forge-desktop.exe" "..\..\..\target\release\forge-desktop.exe"
    File "/oname=forge.exe"         "..\..\..\target\release\forge.exe"

    ; Start menu shortcut.
    CreateDirectory "$SMPROGRAMS\${APPNAME}"
    CreateShortcut  "$SMPROGRAMS\${APPNAME}\${APPNAME}.lnk" "$INSTDIR\forge-desktop.exe"

    ; Uninstaller.
    WriteUninstaller "$INSTDIR\uninstall.exe"
    WriteRegStr HKLM "Software\Microsoft\Windows\CurrentVersion\Uninstall\${APPNAME}" \
        "DisplayName" "${APPNAME}"
    WriteRegStr HKLM "Software\Microsoft\Windows\CurrentVersion\Uninstall\${APPNAME}" \
        "DisplayVersion" "${VERSION}"
    WriteRegStr HKLM "Software\Microsoft\Windows\CurrentVersion\Uninstall\${APPNAME}" \
        "UninstallString" "$\"$INSTDIR\uninstall.exe$\""
    WriteRegStr HKLM "Software\Microsoft\Windows\CurrentVersion\Uninstall\${APPNAME}" \
        "Publisher" "${COMPANY}"
SectionEnd

Section "Uninstall"
    Delete "$INSTDIR\forge-desktop.exe"
    Delete "$INSTDIR\forge.exe"
    Delete "$INSTDIR\uninstall.exe"
    RMDir  "$INSTDIR"
    Delete "$SMPROGRAMS\${APPNAME}\${APPNAME}.lnk"
    RMDir  "$SMPROGRAMS\${APPNAME}"
    DeleteRegKey HKLM "Software\Microsoft\Windows\CurrentVersion\Uninstall\${APPNAME}"
SectionEnd
