Var AppStartMenuFolder
Var ReinstallPageCheck

!include MUI2.nsh
!include FileFunc.nsh
!include x64.nsh
!include WordFunc.nsh

!define MANUFACTURER "{{{manufacturer}}}"
!define PRODUCTNAME "{{{product_name}}}"
!define VERSION "{{{version}}}"
!define INSTALLMODE "{{{install_mode}}}"
!define LICENSE "{{{license}}}"
!define INSTALLERICON "{{{installer_icon}}}"
!define SIDEBARIMAGE "{{{sidebar_image}}}"
!define HEADERIMAGE "{{{header_image}}}"
!define MAINBINARYNAME "{{{main_binary_name}}}"
!define MAINBINARYSRCPATH "{{{main_binary_path}}}"
!define BUNDLEID "{{{bundle_id}}}"
!define OUTFILE "{{{out_file}}}"
!define ARCH "{{{arch}}}"
!define ALLOWDOWNGRADES "{{{allow_downgrades}}}"
!define INSTALLWEBVIEW2MODE "{{{install_webview2_mode}}}"
!define WEBVIEW2INSTALLERARGS "{{{webview2_installer_args}}}"
!define WEBVIEW2BOOTSTRAPPERPATH "{{{webview2_bootstrapper_path}}}"
!define WEBVIEW2INSTALLERPATH "{{{webview2_installer_path}}}"
!define UNINSTKEY "Software\Microsoft\Windows\CurrentVersion\Uninstall\${PRODUCTNAME}"

Name "${PRODUCTNAME}"
OutFile "${OUTFILE}"
Unicode true
SetCompressor /SOLID lzma

!if "${INSTALLMODE}" == "perMachine"
  RequestExecutionLevel highest
!endif

!if "${INSTALLMODE}" == "both"
  !define MULTIUSER_MUI
  !define MULTIUSER_EXECUTIONLEVEL Highest
  !define MULTIUSER_INSTALLMODE_INSTDIR "${PRODUCTNAME}"
  !define MULTIUSER_INSTALLMODE_COMMANDLINE
  !if "${ARCH}" == "x64"
    !define MULTIUSER_USE_PROGRAMFILES64
  !endif
  !define MULTIUSER_INSTALLMODE_DEFAULT_REGISTRY_KEY "${UNINSTKEY}"
  !define MULTIUSER_INSTALLMODE_DEFAULT_REGISTRY_VALUENAME "CurrentUser"
  !define MULTIUSER_INSTALLMODEPAGE_SHOWUSERNAME
  !define MULTIUSER_INSTALLMODE_FUNCTION RestorePreviousInstallLocation
  Function RestorePreviousInstallLocation
    ReadRegStr $4 SHCTX "Software\${MANUFACTURER}\${PRODUCTNAME}" ""
    StrCmp $4 "" +2 0
      StrCpy $INSTDIR $4
  FunctionEnd
  !include MultiUser.nsh
!endif

!if "${INSTALLERICON}" != ""
  !define MUI_ICON "${INSTALLERICON}"
!endif

!if "${SIDEBARIMAGE}" != ""
  !define MUI_WELCOMEFINISHPAGE_BITMAP "${SIDEBARIMAGE}"
!endif

!if "${HEADERIMAGE}" != ""
  !define MUI_HEADERIMAGE
  !define MUI_HEADERIMAGE_BITMAP  "${HEADERIMAGE}"
!endif

; Don't auto jump to finish page after installation page,
; because the installation page has useful info that can be used debug any issues with the installer.
!define MUI_FINISHPAGE_NOAUTOCLOSE
; Use show readme button in the finish page to create a desktop shortcut
!define MUI_FINISHPAGE_SHOWREADME
!define MUI_FINISHPAGE_SHOWREADME_TEXT "Create desktop shortcut"
!define MUI_FINISHPAGE_SHOWREADME_FUNCTION CreateDesktopShortcut
Function CreateDesktopShortcut
  CreateShortcut "$DESKTOP\${MAINBINARYNAME}.lnk" "$INSTDIR\${MAINBINARYNAME}.exe"
  ApplicationID::Set "$DESKTOP\${MAINBINARYNAME}.lnk" "${BUNDLEID}"
FunctionEnd
; Show run app after installation.
!define MUI_FINISHPAGE_RUN "$INSTDIR\${MAINBINARYNAME}.exe"

Function .onInit
  !if "${INSTALLMODE}" == "currentUser"
    SetShellVarContext current
  !else if "${INSTALLMODE}" == "perMachine"
    SetShellVarContext all
  !endif

  !if "${INSTALLMODE}" == "perMachine"
    ; Set default install location
    ${If} ${RunningX64}
      !if "${ARCH}" == "x64"
        StrCpy $INSTDIR "$PROGRAMFILES64\${PRODUCTNAME}"
      !else
        StrCpy $INSTDIR "$PROGRAMFILES\${PRODUCTNAME}"
      !endif
    ${Else}
      StrCpy $INSTDIR "$PROGRAMFILES\${PRODUCTNAME}"
    ${EndIf}
  !else if "${INSTALLMODE}" == "currentUser"
    StrCpy $INSTDIR "$LOCALAPPDATA\${PRODUCTNAME}"
  !endif

  !if "${INSTALLMODE}" == "both"
    !insertmacro MULTIUSER_INIT
  !endif
FunctionEnd

; Installer pages, must be ordered as they appear
!insertmacro MUI_PAGE_WELCOME
!if "${LICENSE}" != ""
  !insertmacro MUI_PAGE_LICENSE "${LICENSE}"
!endif
!if "${INSTALLMODE}" == "both"
  !insertmacro MULTIUSER_PAGE_INSTALLMODE
!endif
Page custom PageReinstall PageLeaveReinstall
Function PageReinstall
  ; Check if there is an existing installation, if not, abort the reinstall page
  ReadRegStr $R0 SHCTX "${UNINSTKEY}" ""
  ReadRegStr $R1 SHCTX "${UNINSTKEY}" "UninstallString"
  ${IfThen} "$R0$R1" == "" ${|} Abort ${|}

  ; Compare this installar version with the existing installation and modify the messages presented to the user accordingly
  StrCpy $R4 "older"
  ReadRegStr $R0 SHCTX "${UNINSTKEY}" "DisplayVersion"
  ${IfThen} $R0 == "" ${|} StrCpy $R4 "unknown" ${|}

  nsis_semvercompare::SemverCompare "${VERSION}" $R0
  Pop $R0
  ; Reinstalling the same version
  ${If} $R0 == 0
    StrCpy $R1 "${PRODUCTNAME} ${VERSION} is already installed. Select the operation you want to perform and click Next to continue."
    StrCpy $R2 "Add/Reinstall components"
    StrCpy $R3 "Uninstall ${PRODUCTNAME}"
    !insertmacro MUI_HEADER_TEXT "Already Installed" "Choose the maintenance option to perform."
    StrCpy $R0 "2"
  ; Upgrading
  ${ElseIf} $R0 == 1
    StrCpy $R1 "An $R4 version of ${PRODUCTNAME} is installed on your system. It's recommended that you uninstall the current version before installing. Select the operation you want to perform and click Next to continue."
    StrCpy $R2 "Uninstall before installing"
    StrCpy $R3 "Do not uninstall"
    !insertmacro MUI_HEADER_TEXT "Already Installed" "Choose how you want to install ${PRODUCTNAME}."
    StrCpy $R0 "1"
  ; Downgrading
  ${ElseIf} $R0 == -1
    StrCpy $R1 "A newer version of ${PRODUCTNAME} is already installed! It is not recommended that you install an older version. If you really want to install this older version, it's better to uninstall the current version first. Select the operation you want to perform and click Next to continue."
    StrCpy $R2 "Uninstall before installing"
    !if "${ALLOWDOWNGRADES}" == "true"
      StrCpy $R3 "Do not uninstall"
    !else
      StrCpy $R3 "Do not uninstall (Downgrading without uninstall is disabled for this installer)"
    !endif
    !insertmacro MUI_HEADER_TEXT "Already Installed" "Choose how you want to install ${PRODUCTNAME}."
    StrCpy $R0 "1"
  ${Else}
    Abort
  ${EndIf}

  nsDialogs::Create 1018
  Pop $R4

  ${NSD_CreateLabel} 0 0 100% 24u $R1
  Pop $R1

  ${NSD_CreateRadioButton} 30u 50u -30u 8u $R2
  Pop $R2
  ${NSD_OnClick} $R2 PageReinstallUpdateSelection

  ${NSD_CreateRadioButton} 30u 70u -30u 8u $R3
  Pop $R3
  ; disable this radio button if downgrades are not allowed
  !if "${ALLOWDOWNGRADES}" == "false"
    EnableWindow $R3 0
  !endif
  ${NSD_OnClick} $R3 PageReinstallUpdateSelection

  ${If} $ReinstallPageCheck != 2
    SendMessage $R2 ${BM_SETCHECK} ${BST_CHECKED} 0
  ${Else}
    SendMessage $R3 ${BM_SETCHECK} ${BST_CHECKED} 0
  ${EndIf}

  ${NSD_SetFocus} $R2

  nsDialogs::Show
FunctionEnd
Function PageReinstallUpdateSelection
  Pop $R1

  ${NSD_GetState} $R2 $R1

  ${If} $R1 == ${BST_CHECKED}
    StrCpy $ReinstallPageCheck 1
  ${Else}
    StrCpy $ReinstallPageCheck 2
  ${EndIf}

FunctionEnd
Function PageLeaveReinstall
  ${NSD_GetState} $R2 $R1

  ; $R0 holds whether we are reinstalling the same version or not
  ; $R0 == "1" -> different versions
  ; $R0 == "2" -> same version
  ;
  ; $R1 holds the radio buttons state. its meaning is dependant on the context
  StrCmp $R0 "1" 0 +2 ; Existing install is not the same version?
    StrCmp $R1 "1" reinst_uninstall reinst_done
  StrCmp $R1 "1" reinst_done ; Same version, skip to add/reinstall components?

  reinst_uninstall:
    ReadRegStr $4 SHCTX "Software\${MANUFACTURER}\${PRODUCTNAME}" ""
    ReadRegStr $R1 SHCTX "${UNINSTKEY}" "UninstallString"

    HideWindow

    ClearErrors
    ExecWait '$R1 _?=$4' $0

    BringToFront

    ${IfThen} ${Errors} ${|} StrCpy $0 2 ${|} ; ExecWait failed, set fake exit code

    ${If} $0 <> 0
    ${OrIf} ${FileExists} "$INSTDIR\${MAINBINARYNAME}.exe"
      ${If} $0 = 1 ; User aborted uninstaller?
        StrCmp $R0 "2" 0 +2 ; Is the existing install the same version?
          Quit ; ...yes, already installed, we are done
        Abort
      ${EndIf}
      MessageBox MB_ICONEXCLAMATION "Unable to uninstall!"
      Abort
    ${Else}
      StrCpy $0 $R1 1
      ${IfThen} $0 == '"' ${|} StrCpy $R1 $R1 -1 1 ${|} ; Strip quotes from UninstallString
      Delete $R1
      RMDir $INSTDIR
    ${EndIf}

  reinst_done:
FunctionEnd
!insertmacro MUI_PAGE_DIRECTORY
!insertmacro MUI_PAGE_STARTMENU Application $AppStartMenuFolder
!insertmacro MUI_PAGE_INSTFILES
!insertmacro MUI_PAGE_FINISH
; Uninstaller pages
!insertmacro MUI_UNPAGE_CONFIRM
!insertmacro MUI_UNPAGE_INSTFILES
;Languages
!insertmacro MUI_LANGUAGE English

Section EarlyChecks
  ; Abort silent installer if downgrades is disabled
  !if "${ALLOWDOWNGRADES}" == "false"
  IfSilent 0 done
    System::Call 'kernel32::AttachConsole(i -1)i.r0'
    ${If} $0 != 0
      System::Call 'kernel32::GetStdHandle(i -11)i.r0'
      System::call 'kernel32::SetConsoleTextAttribute(i r0, i 0x0004)' ; set red color
      FileWrite $0 "A newer version is already installed! Automatic silent downgrades are disabled for this installer.$\nIt is not recommended that you install an older version. If you really want to install this older version, you have to uninstall the current version first.$\n"
    ${EndIf}
    Abort
  done:
  !endif
SectionEnd

Section Webview2
  ; Check if Webview2 is already installed and skip this section
  ${If} ${RunningX64}
    ReadRegStr $4 HKLM "SOFTWARE\WOW6432Node\Microsoft\EdgeUpdate\Clients\{F3017226-FE2A-4295-8BDF-00C3A9A7E4C5}" "pv"
  ${Else}
    ReadRegStr $4 HKLM "SOFTWARE\Microsoft\EdgeUpdate\Clients\{F3017226-FE2A-4295-8BDF-00C3A9A7E4C5}" "pv"
  ${EndIf}
  ReadRegStr $5 HKCU "SOFTWARE\Microsoft\EdgeUpdate\Clients\{F3017226-FE2A-4295-8BDF-00C3A9A7E4C5}" "pv"

  StrCmp $4 "" 0 done
  StrCmp $5 "" 0 done

  ;--------------------------------
  ; Webview2 install modes

  !if "${INSTALLWEBVIEW2MODE}" == "downloadBootstrapper"
    Delete "$TEMP\MicrosoftEdgeWebview2Setup.exe"
    DetailPrint "Downloading Webview2 bootstrapper..."
    NScurl::http GET "https://go.microsoft.com/fwlink/p/?LinkId=2124703" "$TEMP\MicrosoftEdgeWebview2Setup.exe" /CANCEL /END
    Pop $0
    ${If} $0 == "OK"
      DetailPrint "Webview2 bootstrapper downloaded sucessfully"
    ${Else}
      DetailPrint "Error: Downloading Webview2 Failed - $0"
      Abort "Failed to install Webview2. The app can't run without it. Try restarting the installer"
    ${EndIf}
    StrCpy $6 "$TEMP\MicrosoftEdgeWebview2Setup.exe"
    Goto install_webview2
  !endif

  !if "${INSTALLWEBVIEW2MODE}" == "embedBootstrapper"
    CreateDirectory "$INSTDIR\redist"
    File /oname="$INSTDIR\redist\MicrosoftEdgeWebview2Setup.exe" "WEBVIEW2BOOTSTRAPPERPATH"
    DetailPrint "Installing Webview2..."
    StrCpy $6 "$INSTDIR\redist\MicrosoftEdgeWebview2Setup.exe"
    Goto install_webview2
  !endif

  !if "${INSTALLWEBVIEW2MODE}" == "offlineInstaller"
    CreateDirectory "$INSTDIR\redist"
    File /oname="$INSTDIR\redist\MicrosoftEdgeWebView2RuntimeInstaller.exe" "WEBVIEW2INSTALLERPATH"
    DetailPrint "Installing Webview2..."
    StrCpy $6 "$INSTDIR\redist\MicrosoftEdgeWebView2RuntimeInstaller.exe"
    Goto install_webview2
  !endif

  Goto done

  install_webview2:
    DetailPrint "Installing Webview2..."
    ; $6 holds the path to the webview2 installer
    ExecWait "$6 /install ${WEBVIEW2INSTALLERARGS}" $1
    ${If} $1 == 0
      DetailPrint "Webview2 installed sucessfully"
    ${Else}
      DetailPrint "Error: Installing Webview2 Failed with exit code $1"
      Abort "Failed to install Webview2. The app can't run without it. Try restarting the installer"
    ${EndIf}

  done:
SectionEnd

!macro CheckIfAppIsRunning
  nsProcess::_FindProcess "${MAINBINARYNAME}.exe"
  Pop $R0
  ${If} $R0 = 0
    IfSilent silent ui
    silent:
      System::Call 'kernel32::AttachConsole(i -1)i.r0'
      ${If} $0 != 0
        System::Call 'kernel32::GetStdHandle(i -11)i.r0'
        System::call 'kernel32::SetConsoleTextAttribute(i r0, i 0x0004)' ; set red color
        FileWrite $0 "${PRODUCTNAME} is running. Please close it first then try again.$\n"
      ${EndIf}
      Abort
    ui:
      MessageBox MB_OKCANCEL "${PRODUCTNAME} is running$\nClick OK to kill it" IDOK ok IDCANCEL cancel
      ok:
        nsProcess::_KillProcess "${MAINBINARYNAME}.exe"
        Pop $R0
        Sleep 500
        ${If} $R0 = 0
          Goto done
        ${Else}
          Abort "Failed to kill ${PRODUCTNAME}. Please close it first then try again"
        ${EndIf}
      cancel:
        Abort "${PRODUCTNAME} is running. Please close it first then try again"
  ${EndIf}
  done:
!macroend

Section Install
  SetOutPath $INSTDIR

  !insertmacro CheckIfAppIsRunning

  ; Copy main executable
  File "${MAINBINARYSRCPATH}"

  ; Copy resources
  {{#each resources}}
    CreateDirectory "$INSTDIR\\{{this.[0]}}"
    File /a "/oname={{this.[1]}}" "{{@key}}"
  {{/each}}

  ; Copy external binaries
  {{#each binaries}}
    File /a "/oname={{this}}" "{{@key}}"
  {{/each}}

  ; Create uninstaller
  WriteUninstaller "$INSTDIR\uninstall.exe"

  ; Save $INSTDIR in registry for future installations
  WriteRegStr SHCTX "Software\${MANUFACTURER}\${PRODUCTNAME}" "" $INSTDIR

  !if "${INSTALLMODE}" == "both"
    ; Save install mode to be selected by default for the next installation such as updating
    WriteRegStr SHCTX "${UNINSTKEY}" $MultiUser.InstallMode 1

    ; Save install mode to be read by the uninstaller in order to remove the correct
    ; registry key
    FileOpen $4 "$INSTDIR\installmode" w
    FileWrite $4 $MultiUser.InstallMode
    FileClose $4
    SetFileAttributes "$INSTDIR\installmode" HIDDEN|READONLY
  !endif

  ; Registry information for add/remove programs
  WriteRegStr SHCTX "${UNINSTKEY}" "DisplayName" "${PRODUCTNAME}"
  WriteRegStr SHCTX "${UNINSTKEY}" "DisplayIcon" "$\"$INSTDIR\${MAINBINARYNAME}.exe$\""
  WriteRegStr SHCTX "${UNINSTKEY}" "DisplayVersion" "${VERSION}"
  WriteRegStr SHCTX "${UNINSTKEY}" "Publisher" "${MANUFACTURER}"
  WriteRegStr SHCTX "${UNINSTKEY}" "InstallLocation" "$\"$INSTDIR$\""
  WriteRegStr SHCTX "${UNINSTKEY}" "UninstallString" "$\"$INSTDIR\uninstall.exe$\""
  WriteRegDWORD SHCTX "${UNINSTKEY}" "NoModify" "1"
  WriteRegDWORD SHCTX "${UNINSTKEY}" "NoRepair" "1"
  ${GetSize} "$INSTDIR" "/S=0K" $0 $1 $2
  IntFmt $0 "0x%08X" $0
  WriteRegDWORD SHCTX "${UNINSTKEY}" "EstimatedSize" "$0"

  ; Create start menu shortcut
  !insertmacro MUI_STARTMENU_WRITE_BEGIN Application
    CreateDirectory "$SMPROGRAMS\$AppStartMenuFolder"
    CreateShortcut "$SMPROGRAMS\$AppStartMenuFolder\${MAINBINARYNAME}.lnk" "$INSTDIR\${MAINBINARYNAME}.exe"
    ApplicationID::Set "$SMPROGRAMS\$AppStartMenuFolder\${MAINBINARYNAME}.lnk" "${BUNDLEID}"
  !insertmacro MUI_STARTMENU_WRITE_END

SectionEnd

Function un.onInit
  !if "${INSTALLMODE}" == "both"
    !insertmacro MULTIUSER_UNINIT
  !endif
FunctionEnd

Section Uninstall
  !insertmacro CheckIfAppIsRunning

  ; Remove registry information for add/remove programs
  !if "${INSTALLMODE}" == "both"
    ; Get the saved install mode
    FileOpen $4 "$INSTDIR\installmode" r
    FileRead $4 $1
    FileClose $4
    Delete "$INSTDIR\installmode"

    ${If} $1 == "AllUsers"
      DeleteRegKey HKLM "${UNINSTKEY}"
    ${ElseIf} $1 == "CurrentUser"
      DeleteRegKey HKCU "${UNINSTKEY}"
    ${EndIf}
  !else if "${INSTALLMODE}" == "perMachine"
    DeleteRegKey HKLM "${UNINSTKEY}"
  !else
    DeleteRegKey HKCU "${UNINSTKEY}"
  !endif

  ; Delete the app directory and its content from disk
  ; Copy main executable
  Delete "$INSTDIR\${MAINBINARYNAME}.exe"

  ; Delete resources
  {{#each resources}}
    Delete "$INSTDIR\\{{this.[1]}}"
    RMDir "$INSTDIR\\{{this.[0]}}"
  {{/each}}

  ; Delete external binaries
  {{#each binaries}}
    Delete "$INSTDIR\\{{this}}"
  {{/each}}

  ; Delete uninstaller
  Delete "$INSTDIR\uninstall.exe"

  RMDir "$INSTDIR"

  ; Remove start menu shortcut
  !insertmacro MUI_STARTMENU_GETFOLDER Application $AppStartMenuFolder
  Delete "$SMPROGRAMS\$AppStartMenuFolder\${MAINBINARYNAME}.lnk"
  RMDir "$SMPROGRAMS\$AppStartMenuFolder"

  ; Remove desktop shortcuts
  Delete "$DESKTOP\${MAINBINARYNAME}.lnk"
SectionEnd
