#!/bin/bash
# 卸载前备份用户数据库文件（兼容 deb prerm 与 rpm %preun 参数约定）
# deb prerm:  $1 ∈ {remove, upgrade, failed-upgrade, deconfigure}
# rpm %preun: $1 ∈ {0=uninstall, 1=upgrade}

set -e

APP_DATA_DIR="${HOME}/.local/share/com.gebinee.database-editor"
DB_PATH_FILE="${APP_DATA_DIR}/db_path.txt"
BACKUP_DIR="/tmp/gebinee_db_editor_backup"

# 判断是否为真正卸载（非升级）
IS_REMOVE=0
if [ "$1" = "remove" ] || [ "$1" = "0" ]; then
    IS_REMOVE=1
fi

if [ "$IS_REMOVE" = "0" ]; then
    exit 0
fi

# 清理可能残留的旧备份，重建空目录
rm -rf "${BACKUP_DIR}"
mkdir -p "${BACKUP_DIR}"

# 读取 db_path.txt
if [ -f "${DB_PATH_FILE}" ]; then
    DB_PATH=$(tr -d '[:space:]' < "${DB_PATH_FILE}")

    if [ -n "${DB_PATH}" ] && [ -f "${DB_PATH}" ]; then
        # 备份主 DB 文件
        cp "${DB_PATH}" "${BACKUP_DIR}/db.sqlite"

        # 备份 WAL（若存在）
        [ -f "${DB_PATH}-wal" ] && cp "${DB_PATH}-wal" "${BACKUP_DIR}/db.sqlite-wal"

        # 备份 SHM（若存在）
        [ -f "${DB_PATH}-shm" ] && cp "${DB_PATH}-shm" "${BACKUP_DIR}/db.sqlite-shm"

        # 保存原始路径供 postrm 判断恢复
        printf '%s' "${DB_PATH}" > "${BACKUP_DIR}/original_path.txt"
    fi
fi

exit 0
