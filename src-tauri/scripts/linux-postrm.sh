#!/bin/bash
# 卸载后删除 app_data_dir 并按需恢复 DB（兼容 deb postrm 与 rpm %postun 参数约定）
# deb postrm:  $1 ∈ {remove, purge, upgrade, failed-upgrade, abort-install, abort-upgrade}
# rpm %postun: $1 ∈ {0=uninstall, 1=upgrade}

set -e

APP_DATA_DIR="${HOME}/.local/share/com.gebinee.database-editor"
BACKUP_DIR="/tmp/gebinee_db_editor_backup"

# 判断是否为真正卸载（非升级）
IS_REMOVE=0
if [ "$1" = "remove" ] || [ "$1" = "purge" ] || [ "$1" = "0" ]; then
    IS_REMOVE=1
fi

if [ "$IS_REMOVE" = "0" ]; then
    exit 0
fi

# 删除 app_data_dir（用户数据清理）
if [ -d "${APP_DATA_DIR}" ]; then
    rm -rf "${APP_DATA_DIR}"
fi

# 若已备份 DB，且原路径已不存在（说明 DB 原本位于 app_data_dir 内被一并删除），则恢复
if [ -d "${BACKUP_DIR}" ] && [ -f "${BACKUP_DIR}/original_path.txt" ]; then
    ORIGINAL_PATH=$(cat "${BACKUP_DIR}/original_path.txt")

    if [ -n "${ORIGINAL_PATH}" ] && [ ! -e "${ORIGINAL_PATH}" ]; then
        # 原路径已被删除，需恢复
        PARENT_DIR=$(dirname "${ORIGINAL_PATH}")
        mkdir -p "${PARENT_DIR}"

        [ -f "${BACKUP_DIR}/db.sqlite" ] && cp "${BACKUP_DIR}/db.sqlite" "${ORIGINAL_PATH}"
        [ -f "${BACKUP_DIR}/db.sqlite-wal" ] && cp "${BACKUP_DIR}/db.sqlite-wal" "${ORIGINAL_PATH}-wal"
        [ -f "${BACKUP_DIR}/db.sqlite-shm" ] && cp "${BACKUP_DIR}/db.sqlite-shm" "${ORIGINAL_PATH}-shm"
    fi

    # 清理备份
    rm -rf "${BACKUP_DIR}"
fi

exit 0
