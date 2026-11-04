; Little Joan as a browser the user may choose for web links (LJ45), per user (the installer is
; currentUser): Windows lists it in Settings > Default apps from these keys. Little Joan never
; makes itself the default: no Software\Classes\http or \https, never UserChoice (Windows protects
; it anyway). Links arrive as `little-joan.exe --link "<url>"`, and `--link` takes exactly the
; argument after it, whatever it holds (src-tauri/src/cli.rs). The uninstaller removes exactly what
; is written here (not while updating, when the new installer writes it again), and only when the
; ProgId's command is still this install's own (as Tauri's deep-link removal checks): another copy
; of Little Joan installed elsewhere keeps its registration.
;
; Included by Tauri's installer.nsi (bundle.windows.nsis.installerHooks); MAINBINARYNAME and
; PRODUCTNAME are its own defines, UpdateMode its own variable.

!define LJ_CLIENT_KEY "Software\Clients\StartMenuInternet\LittleJoan"
!define LJ_PROGID_KEY "Software\Classes\LittleJoanURL"
!define LJ_REGISTERED_KEY "Software\RegisteredApplications"

!macro NSIS_HOOK_POSTINSTALL
  WriteRegStr HKCU "${LJ_CLIENT_KEY}" "" "${PRODUCTNAME}"
  WriteRegStr HKCU "${LJ_CLIENT_KEY}\Capabilities" "ApplicationName" "${PRODUCTNAME}"
  WriteRegStr HKCU "${LJ_CLIENT_KEY}\Capabilities" "ApplicationDescription" "A tiny pop-up browser that hibernates until you call it."
  WriteRegStr HKCU "${LJ_CLIENT_KEY}\Capabilities" "ApplicationIcon" "$INSTDIR\${MAINBINARYNAME}.exe,0"
  WriteRegStr HKCU "${LJ_CLIENT_KEY}\Capabilities\URLAssociations" "http" "LittleJoanURL"
  WriteRegStr HKCU "${LJ_CLIENT_KEY}\Capabilities\URLAssociations" "https" "LittleJoanURL"
  WriteRegStr HKCU "${LJ_CLIENT_KEY}\DefaultIcon" "" "$INSTDIR\${MAINBINARYNAME}.exe,0"
  WriteRegStr HKCU "${LJ_CLIENT_KEY}\shell\open\command" "" "$\"$INSTDIR\${MAINBINARYNAME}.exe$\""
  WriteRegStr HKCU "${LJ_REGISTERED_KEY}" "${PRODUCTNAME}" "${LJ_CLIENT_KEY}\Capabilities"
  WriteRegStr HKCU "${LJ_PROGID_KEY}" "" "${PRODUCTNAME} URL"
  WriteRegStr HKCU "${LJ_PROGID_KEY}\DefaultIcon" "" "$INSTDIR\${MAINBINARYNAME}.exe,0"
  WriteRegStr HKCU "${LJ_PROGID_KEY}\shell\open\command" "" "$\"$INSTDIR\${MAINBINARYNAME}.exe$\" --link $\"%1$\""
  ; SHCNE_ASSOCCHANGED, SHCNF_IDLIST: the shell reads the associations again.
  System::Call 'shell32::SHChangeNotify(i 0x08000000, i 0, p 0, p 0)'
!macroend

!macro NSIS_HOOK_POSTUNINSTALL
  ${If} $UpdateMode <> 1
    ReadRegStr $R7 HKCU "${LJ_PROGID_KEY}\shell\open\command" ""
    ${If} $R7 == "$\"$INSTDIR\${MAINBINARYNAME}.exe$\" --link $\"%1$\""
      DeleteRegKey HKCU "${LJ_CLIENT_KEY}"
      DeleteRegValue HKCU "${LJ_REGISTERED_KEY}" "${PRODUCTNAME}"
      DeleteRegKey HKCU "${LJ_PROGID_KEY}"
      System::Call 'shell32::SHChangeNotify(i 0x08000000, i 0, p 0, p 0)'
    ${EndIf}
  ${EndIf}
!macroend
