use sudo_test::{Command, Env, TextFile};

use crate::{
    visudo::{ETC_DIR, ETC_SUDOERS},
    Result, SUDOERS_ALL_ALL_NOPASSWD, USERNAME,
};

use super::TMP_SUDOERS;

const DEFAULT_CHMOD: &str = "440";

#[test]
fn no_syntax_errors_and_ok_ownership_and_perms() -> Result<()> {
    let env = Env(TextFile("").chmod(DEFAULT_CHMOD)).build()?;

    let output = Command::new("visudo").arg("-c").output(&env)?;

    assert!(output.status().success(), "{}", output.stderr());
    assert!(output.stderr().is_empty());
    assert_eq!(format!("{ETC_DIR}/sudoers: parsed OK"), output.stdout()?);

    Ok(())
}

#[test]
fn bad_perms() -> Result<()> {
    let env = Env(TextFile("").chmod("444")).build()?;

    let output = Command::new("visudo").arg("-c").output(&env)?;

    assert!(!output.status().success());
    assert_eq!(Some(1), output.status().code());
    assert_contains!(
        output.stderr(),
        format!("{ETC_DIR}/sudoers: bad permissions, should be mode 0440")
    );

    Ok(())
}

#[test]
fn bad_ownership() -> Result<()> {
    let env = Env(TextFile("").chown(USERNAME).chmod(DEFAULT_CHMOD))
        .user(USERNAME)
        .build()?;

    let output = Command::new("visudo").arg("-c").output(&env)?;

    assert!(!output.status().success());
    assert_eq!(Some(1), output.status().code());
    assert_contains!(
        output.stderr(),
        format!("{ETC_DIR}/sudoers: wrong owner (uid, gid) should be (0, 0)")
    );

    Ok(())
}

#[test]
fn bad_syntax() -> Result<()> {
    let env = Env(TextFile("this is fine").chmod(DEFAULT_CHMOD)).build()?;

    let output = Command::new("visudo").arg("-c").output(&env)?;

    assert!(!output.status().success());
    assert_eq!(Some(1), output.status().code());
    assert_contains!(output.stderr(), "syntax error");

    Ok(())
}

#[test]
fn file_does_not_exist() -> Result<()> {
    let env = Env("").build()?;

    Command::new("rm")
        .args(["-f", ETC_SUDOERS])
        .output(&env)?
        .assert_success()?;

    let output = Command::new("visudo").arg("-c").output(&env)?;

    assert!(!output.status().success());
    assert_eq!(Some(1), output.status().code());
    assert_contains!(
        output.stderr(),
        format!("visudo: unable to open {ETC_DIR}/sudoers: No such file or directory")
    );

    Ok(())
}

#[test]
#[ignore = "gh657"]
fn flag_quiet_ok() -> Result<()> {
    let env = Env(TextFile("").chmod(DEFAULT_CHMOD)).build()?;

    let output = Command::new("visudo").args(["-c", "-q"]).output(&env)?;

    assert!(output.status().success());
    assert!(output.stderr().is_empty());
    assert!(output.stdout()?.is_empty());

    Ok(())
}

#[test]
#[ignore = "gh657"]
fn flag_quiet_bad_perms() -> Result<()> {
    let env = Env(TextFile("").chmod("444")).build()?;

    let output = Command::new("visudo").args(["-c", "-q"]).output(&env)?;

    assert!(!output.status().success());
    assert_eq!(Some(1), output.status().code());
    assert!(output.stderr().is_empty());

    Ok(())
}

#[test]
#[ignore = "gh657"]
fn flag_quiet_bad_ownership() -> Result<()> {
    let env = Env(TextFile("").chmod(DEFAULT_CHMOD).chown(USERNAME))
        .user(USERNAME)
        .build()?;

    let output = Command::new("visudo").args(["-c", "-q"]).output(&env)?;

    assert!(!output.status().success());
    assert_eq!(Some(1), output.status().code());
    assert!(output.stderr().is_empty());

    Ok(())
}

#[test]
#[ignore = "gh657"]
fn flag_quiet_bad_syntax() -> Result<()> {
    let env = Env(TextFile("this is fine").chmod(DEFAULT_CHMOD)).build()?;

    let output = Command::new("visudo").args(["-c", "-q"]).output(&env)?;

    assert!(!output.status().success());
    assert_eq!(Some(1), output.status().code());
    assert!(output.stderr().is_empty());

    Ok(())
}

#[test]
fn flag_file() -> Result<()> {
    let file_path = TMP_SUDOERS;
    let env = Env("this is fine")
        .file(file_path, "")
        .user(USERNAME)
        .build()?;

    Command::new("visudo")
        .args(["--check", "--file", file_path])
        .output(&env)?
        .assert_success()
}

#[test]
fn flag_file_bad_syntax() -> Result<()> {
    let file_path = TMP_SUDOERS;
    let env = Env("")
        .file(file_path, "this is fine")
        .user(USERNAME)
        .build()?;

    let output = Command::new("visudo")
        .args(["--check", "--file", file_path])
        .output(&env)?;

    assert!(!output.status().success());
    assert_eq!(Some(1), output.status().code());

    assert_contains!(output.stderr(), "syntax error");

    Ok(())
}

#[test]
fn flag_file_does_not_check_perms_nor_ownership() -> Result<()> {
    let file_path = TMP_SUDOERS;
    let env = Env("")
        .file(
            file_path,
            TextFile("").chown(format!("{USERNAME}:users")).chmod("777"),
        )
        .user(USERNAME)
        .build()?;

    Command::new("visudo")
        .args(["--check", "--file", file_path])
        .output(&env)?
        .assert_success()
}

#[test]
#[ignore = "gh657"]
fn stdin() -> Result<()> {
    let env = Env("").build()?;

    Command::new("visudo")
        .args(["-c", "-"])
        .stdin(SUDOERS_ALL_ALL_NOPASSWD)
        .output(&env)?
        .assert_success()
}

#[test]
#[ignore = "gh657"]
fn stdin_bad_syntax() -> Result<()> {
    let env = Env("").build()?;

    let output = Command::new("visudo")
        .args(["-c", "-"])
        .stdin("this is fine")
        .output(&env)?;

    assert!(!output.status().success());
    assert_eq!(Some(1), output.status().code());
    assert_contains!(output.stderr(), "syntax error");

    Ok(())
}
