; NSIS 卸载钩子：卸载时保留数据库文件，删除其余应用数据
; app_data_dir = %APPDATA%\<identifier>，对应 Tauri 的 app_data_dir
; 数据库路径由 Rust 端写入 db_path.txt（纯文本，单行），供此钩子读取

Var DbOriginalPath
Var DbBackupPath
Var DbBackupWal
Var DbBackupShm

; 去除行尾换行符（\r \n）
Function un.TrimNewlines
  Exch $0
  Push $1
  loop:
    StrCpy $1 $0 1 -1
    StrCmp $1 "$\r" trim
    StrCmp $1 "$\n" trim
    Goto done
  trim:
    StrCpy $0 $0 -1
    Goto loop
  done:
    Pop $1
    Exch $0
FunctionEnd

; 卸载前：读取 db_path.txt，暂存数据库文件及 WAL/SHM 到 $TEMP
!macro NSIS_HOOK_PREUNINSTALL
  ReadEnvStr $0 "APPDATA"
  StrCpy $1 "$0\${BUNDLEID}"
  StrCpy $DbOriginalPath ""
  StrCpy $DbBackupPath ""
  StrCpy $DbBackupWal ""
  StrCpy $DbBackupShm ""
  ; 读取 db_path.txt
  IfFileExists "$1\db_path.txt" 0 preun_done
  ClearErrors
  FileOpen $2 "$1\db_path.txt" r
  IfErrors preun_done
  FileRead $2 $DbOriginalPath
  FileClose $2
  ; 去掉行尾换行符
  Push $DbOriginalPath
  Call un.TrimNewlines
  Pop $DbOriginalPath
  StrCmp "$DbOriginalPath" "" preun_done 0
  ; 暂存数据库主文件
  IfFileExists "$DbOriginalPath" 0 preun_done
  StrCpy $DbBackupPath "$TEMP\db_editor_backup.sqlite"
  CopyFiles /SILENT "$DbOriginalPath" "$DbBackupPath"
  ; 暂存 WAL 文件
  IfFileExists "$DbOriginalPath-wal" 0 +3
    StrCpy $DbBackupWal "$DbBackupPath-wal"
    CopyFiles /SILENT "$DbOriginalPath-wal" "$DbBackupWal"
  ; 暂存 SHM 文件
  IfFileExists "$DbOriginalPath-shm" 0 +3
    StrCpy $DbBackupShm "$DbBackupPath-shm"
    CopyFiles /SILENT "$DbOriginalPath-shm" "$DbBackupShm"
  preun_done:
!macroend

; 卸载后：若原数据库文件已被删除（位于 app_data_dir 内），从 $TEMP 恢复；最后清理暂存
!macro NSIS_HOOK_POSTUNINSTALL
  StrCmp "$DbOriginalPath" "" postun_done 0
  StrCmp "$DbBackupPath" "" postun_done 0
  ; 仅当原文件已不存在时才恢复（若数据库在 app_data_dir 外，未被删除，则跳过恢复）
  IfFileExists "$DbOriginalPath" postun_cleanup 0
  CopyFiles /SILENT "$DbBackupPath" "$DbOriginalPath"
  IfFileExists "$DbBackupWal" 0 +2
    CopyFiles /SILENT "$DbBackupWal" "$DbOriginalPath-wal"
  IfFileExists "$DbBackupShm" 0 +2
    CopyFiles /SILENT "$DbBackupShm" "$DbOriginalPath-shm"
  postun_cleanup:
  Delete "$DbBackupPath"
  Delete "$DbBackupWal"
  Delete "$DbBackupShm"
  postun_done:
!macroend
