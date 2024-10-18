use std::cmp::Ordering;
use std::fmt;
use std::process::exit;

use libc::{c_char, chdir, chroot, fork, getpwnam, getuid, setgid, setsid, setuid, umask};

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Clone)]
pub enum DaemonizeError {
    Fork,
    SetSession,
    SetGroup,
    SetUser,
    Chroot,
    Chdir,
}

impl fmt::Display for DaemonizeError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match *self {
            DaemonizeError::Fork => "unable to fork",
            DaemonizeError::SetSession => "unable to create new process session",
            DaemonizeError::SetGroup => "unable to set group (drop privileges)",
            DaemonizeError::SetUser => "unable to set user (drop privileges)",
            DaemonizeError::Chroot => "unable to enter chroot jail",
            DaemonizeError::Chdir => "failed to change directory",
        }
        .fmt(f)
    }
}

/// Форкает и завершает работу текущего приложения
fn fork_and_exit() -> Result<(), DaemonizeError> {
    // Системный вызов fork
    let pid = unsafe { fork() };
    match pid.cmp(&0) {
        // Если отрицательный новый id процесса, тогда выдаем ошибку
        Ordering::Less => Err(DaemonizeError::Fork),
        // Если нулевой, значит мы в нашем новом процессе
        Ordering::Equal => Ok(()),
        // Если нам вернулся ID дочернего процесса - выходим без ошибок
        Ordering::Greater => exit(0),
    }
}

pub fn daemonize() -> Result<(), DaemonizeError> {
    // Форкаем и завершаем текущее приложение
    fork_and_exit()?;

    // Избавляемся от уничтожения чилда, когда родитель уничтожается
    // Таким образом процесс становится открепленным
    if unsafe { setsid() } < 0 {
        return Err(DaemonizeError::SetSession);
    }

    // Заново форкаем текщего чилда для создания полностью окрепленного процесса
    fork_and_exit()
}

/// Сменяем права пользователя на nobody для безопасности
pub fn drop_privileges() -> Result<(), DaemonizeError> {
    // Получаем значение пароля из /etc/passwd, передаем сишную строку с нулем в конце
    let usr = unsafe { getpwnam("nobody\x00".as_ptr() as *const c_char) };
    if usr.is_null() {
        return Err(DaemonizeError::SetGroup);
    }

    // Сменяем корневую директорию если текущий пользователь нулевой
    let uid = unsafe { getuid() };
    if uid == 0 && unsafe { chroot("/tmp\x00".as_ptr() as *const c_char) } != 0 {
        return Err(DaemonizeError::Chroot);
    }

    // Для новых файликов будет нулевая маска
    unsafe { umask(0) };

    // Изменяем рабочую директорию на корень
    if unsafe { chdir("/\x00".as_ptr() as *const c_char) } != 0 {
        return Err(DaemonizeError::Chdir);
    }

    // Заменяем группу на nobody
    if unsafe { setgid((*usr).pw_gid) } != 0 {
        return Err(DaemonizeError::SetGroup);
    }

    // Заменяем пользователя на nobody
    if unsafe { setuid((*usr).pw_uid) } != 0 {
        Err(DaemonizeError::SetUser)
    } else {
        Ok(())
    }
}
