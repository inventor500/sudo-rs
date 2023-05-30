mod check;
mod keep;

use core::fmt;

use sudo_test::{Command, Env};

use crate::{helpers, Result, SUDOERS_ALL_ALL_NOPASSWD, SUDO_ENV_DEFAULT_PATH};

enum EnvList {
    #[allow(dead_code)]
    Check,
    Keep,
}

impl fmt::Display for EnvList {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let s = match self {
            EnvList::Check => "env_check",
            EnvList::Keep => "env_keep",
        };
        f.write_str(s)
    }
}

fn equal_single(env_list: EnvList) -> Result<()> {
    let env_name = "SHOULD_BE_PRESERVED";
    let env_val = "42";
    let env = Env([
        SUDOERS_ALL_ALL_NOPASSWD,
        &format!("Defaults {env_list} = {env_name}"),
    ])
    .build()?;

    let stdout = Command::new("env")
        .arg(format!("{env_name}={env_val}"))
        .args(["sudo", "env"])
        .exec(&env)?
        .stdout()?;
    let sudo_env = helpers::parse_env_output(&stdout)?;

    assert_eq!(Some(env_val), sudo_env.get(env_name).copied());

    Ok(())
}

fn equal_multiple(env_list: EnvList) -> Result<()> {
    let env_name1 = "SHOULD_BE_PRESERVED";
    let env_name2 = "ALSO_SHOULD_BE_PRESERVED";
    let env_val1 = "42";
    let env_val2 = "24";
    let env = Env([
        SUDOERS_ALL_ALL_NOPASSWD,
        &format!("Defaults {env_list} = \"{env_name1} {env_name2}\""),
    ])
    .build()?;

    let stdout = Command::new("env")
        .arg(format!("{env_name1}={env_val1}"))
        .arg(format!("{env_name2}={env_val2}"))
        .args(["sudo", "env"])
        .exec(&env)?
        .stdout()?;
    let sudo_env = helpers::parse_env_output(&stdout)?;

    assert_eq!(Some(env_val1), sudo_env.get(env_name1).copied());
    assert_eq!(Some(env_val2), sudo_env.get(env_name2).copied());

    Ok(())
}

fn equal_repeated(env_list: EnvList) -> Result<()> {
    let env_name = "SHOULD_BE_PRESERVED";
    let env_val = "42";
    let env = Env([
        SUDOERS_ALL_ALL_NOPASSWD,
        &format!("Defaults {env_list} = \"{env_name} {env_name}\""),
    ])
    .build()?;

    let stdout = Command::new("env")
        .arg(format!("{env_name}={env_val}"))
        .args(["sudo", "env"])
        .exec(&env)?
        .stdout()?;
    let sudo_env = helpers::parse_env_output(&stdout)?;

    assert_eq!(Some(env_val), sudo_env.get(env_name).copied());

    Ok(())
}

fn equal_overrides(env_list: EnvList) -> Result<()> {
    let env_name1 = "SHOULD_BE_PRESERVED";
    let env_val1 = "42";
    let env_name2 = "SHOULD_BE_REMOVED";
    let env_val2 = "24";
    let env = Env([
        SUDOERS_ALL_ALL_NOPASSWD,
        &format!("Defaults {env_list} = \"{env_name1} {env_name2}\""),
        &format!("Defaults {env_list} = {env_name1}"),
    ])
    .build()?;

    let stdout = Command::new("env")
        .arg(format!("{env_name1}={env_val1}"))
        .arg(format!("{env_name2}={env_val2}"))
        .args(["sudo", "env"])
        .exec(&env)?
        .stdout()?;
    let sudo_env = helpers::parse_env_output(&stdout)?;

    assert!(sudo_env.get(env_name2).is_none());
    assert_eq!(Some(env_val1), sudo_env.get(env_name1).copied());

    Ok(())
}

fn plus_equal_on_empty_set(env_list: EnvList) -> Result<()> {
    let env_name = "SHOULD_BE_PRESERVED";
    let env_value = "42";
    let env = Env([
        SUDOERS_ALL_ALL_NOPASSWD,
        &format!("Defaults {env_list} += {env_name}"),
    ])
    .build()?;

    let stdout = Command::new("env")
        .arg(format!("{env_name}={env_value}"))
        .args(["sudo", "env"])
        .exec(&env)?
        .stdout()?;
    let sudo_env = helpers::parse_env_output(&stdout)?;

    assert_eq!(Some(env_value), sudo_env.get(env_name).copied());

    Ok(())
}

fn plus_equal_appends(env_list: EnvList) -> Result<()> {
    let env_name1 = "SHOULD_BE_PRESERVED";
    let env_name2 = "ALSO_SHOULD_BE_PRESERVED";
    let env_val1 = "42";
    let env_val2 = "24";
    let env = Env([
        SUDOERS_ALL_ALL_NOPASSWD,
        &format!("Defaults {env_list} = {env_name1}"),
        &format!("Defaults {env_list} += {env_name2}"),
    ])
    .build()?;

    let stdout = Command::new("env")
        .arg(format!("{env_name1}={env_val1}"))
        .arg(format!("{env_name2}={env_val2}"))
        .args(["sudo", "env"])
        .exec(&env)?
        .stdout()?;
    let sudo_env = helpers::parse_env_output(&stdout)?;

    assert_eq!(Some(env_val1), sudo_env.get(env_name1).copied());
    assert_eq!(Some(env_val2), sudo_env.get(env_name2).copied());

    Ok(())
}

fn plus_equal_repeated(env_list: EnvList) -> Result<()> {
    let env_name = "SHOULD_BE_PRESERVED";
    let env_val = "42";
    let env = Env([
        SUDOERS_ALL_ALL_NOPASSWD,
        &format!("Defaults {env_list} = {env_name}"),
        &format!("Defaults {env_list} += {env_name}"),
    ])
    .build()?;

    let stdout = Command::new("env")
        .arg(format!("{env_name}={env_val}"))
        .args(["sudo", "env"])
        .exec(&env)?
        .stdout()?;
    let sudo_env = helpers::parse_env_output(&stdout)?;

    assert_eq!(Some(env_val), sudo_env.get(env_name).copied());

    Ok(())
}

// see 'environment' section in `man sudo`
// the variables HOME, LOGNAME, MAIL and USER are set by sudo with a value that depends on the
// target user *unless* they appear in the env_keep list
fn vars_with_target_user_specific_values(env_list: EnvList) -> Result<()> {
    let home = "my-home";
    let logname = "my-logname";
    let mail = "my-mail";
    let user = "my-user";

    let env = Env([
        SUDOERS_ALL_ALL_NOPASSWD,
        &format!("Defaults {env_list} = \"HOME LOGNAME MAIL USER\""),
    ])
    .build()?;

    let stdout = Command::new("env")
        .arg(format!("HOME={home}"))
        .arg(format!("LOGNAME={logname}"))
        .arg(format!("MAIL={mail}"))
        .arg(format!("USER={user}"))
        .args(["sudo", "env"])
        .exec(&env)?
        .stdout()?;
    let sudo_env = helpers::parse_env_output(&stdout)?;

    assert_eq!(Some(home), sudo_env.get("HOME").copied());
    assert_eq!(Some(logname), sudo_env.get("LOGNAME").copied());
    assert_eq!(Some(mail), sudo_env.get("MAIL").copied());
    assert_eq!(Some(user), sudo_env.get("USER").copied());

    Ok(())
}

// these variables cannot be preserved as they'll be set by sudo
fn sudo_env_vars(env_list: EnvList) -> Result<()> {
    let env = Env([
        SUDOERS_ALL_ALL_NOPASSWD,
        &format!("Defaults {env_list} = \"SUDO_COMMAND SUDO_GID SUDO_UID SUDO_USER\""),
    ])
    .build()?;

    let stdout = Command::new("env")
        .arg("SUDO_COMMAND=command")
        .arg("SUDO_GID=gid")
        .arg("SUDO_UID=uid")
        .arg("SUDO_USER=user")
        .args(["sudo", "env"])
        .exec(&env)?
        .stdout()?;
    let sudo_env = helpers::parse_env_output(&stdout)?;

    assert_eq!(Some("/usr/bin/env"), sudo_env.get("SUDO_COMMAND").copied());
    assert_eq!(Some("0"), sudo_env.get("SUDO_GID").copied());
    assert_eq!(Some("0"), sudo_env.get("SUDO_UID").copied());
    assert_eq!(Some("root"), sudo_env.get("SUDO_USER").copied());

    Ok(())
}

fn user_set_to_preserved_logname_value(env_list: EnvList) -> Result<()> {
    let value = "ghost";
    let env = Env([
        SUDOERS_ALL_ALL_NOPASSWD,
        &format!("Defaults {env_list} = \"LOGNAME\""),
    ])
    .build()?;

    let stdout = Command::new("env")
        .arg(format!("LOGNAME={value}"))
        .args(["sudo", "env"])
        .exec(&env)?
        .stdout()?;
    let sudo_env = helpers::parse_env_output(&stdout)?;

    assert_eq!(Some(value), sudo_env.get("LOGNAME").copied());
    assert_eq!(Some(value), sudo_env.get("USER").copied());

    Ok(())
}

fn logname_set_to_preserved_user_value(env_list: EnvList) -> Result<()> {
    let value = "ghost";
    let env = Env([
        SUDOERS_ALL_ALL_NOPASSWD,
        &format!("Defaults {env_list} = \"USER\""),
    ])
    .build()?;

    let stdout = Command::new("env")
        .arg(format!("USER={value}"))
        .args(["sudo", "env"])
        .exec(&env)?
        .stdout()?;
    let sudo_env = helpers::parse_env_output(&stdout)?;

    assert_eq!(Some(value), sudo_env.get("LOGNAME").copied());
    assert_eq!(Some(value), sudo_env.get("USER").copied());

    Ok(())
}

fn if_value_starts_with_parentheses_variable_is_removed(env_list: EnvList) -> Result<()> {
    let env_name = "SHOULD_BE_REMOVED";
    let env_val = "() 42";
    let env = Env([
        SUDOERS_ALL_ALL_NOPASSWD,
        &format!("Defaults {env_list} = {env_name}"),
    ])
    .build()?;

    let stdout = Command::new("env")
        .arg(format!("{env_name}={env_val}"))
        .args(["sudo", "env"])
        .exec(&env)?
        .stdout()?;
    let sudo_env = helpers::parse_env_output(&stdout)?;

    assert!(sudo_env.get(env_name).is_none());

    Ok(())
}

fn key_value_matches(env_list: EnvList) -> Result<()> {
    let env_name = "KEY";
    let env_val = "value";
    let env = Env([
        SUDOERS_ALL_ALL_NOPASSWD,
        &format!("Defaults {env_list} = \"{env_name}={env_val}\""),
    ])
    .build()?;

    let stdout = Command::new("env")
        .arg(format!("{env_name}={env_val}"))
        .args(["sudo", "env"])
        .exec(&env)?
        .stdout()?;
    let sudo_env = helpers::parse_env_output(&stdout)?;

    assert_eq!(Some(env_val), sudo_env.get(env_name).copied());

    Ok(())
}

fn key_value_no_match(env_list: EnvList) -> Result<()> {
    let env_name = "KEY";
    let env_val = "value";
    let env = Env([
        SUDOERS_ALL_ALL_NOPASSWD,
        &format!("Defaults {env_list} = \"{env_name}={env_val}\""),
    ])
    .build()?;

    let stdout = Command::new("env")
        .arg(format!("{env_name}=different-value"))
        .args(["sudo", "env"])
        .exec(&env)?
        .stdout()?;
    let sudo_env = helpers::parse_env_output(&stdout)?;

    assert_eq!(None, sudo_env.get(env_name));

    Ok(())
}

// without the double quotes the RHS is not interpreted as a key value pair
// also see the `key_value_matches` test
fn key_value_syntax_needs_double_quotes(env_list: EnvList) -> Result<()> {
    let env_name = "KEY";
    let env_val = "value";
    let env = Env([
        SUDOERS_ALL_ALL_NOPASSWD,
        &format!("Defaults {env_list} = {env_name}={env_val}"),
    ])
    .build()?;

    let stdout = Command::new("env")
        .arg(format!("{env_name}={env_val}"))
        .args(["sudo", "env"])
        .exec(&env)?
        .stdout()?;
    let sudo_env = helpers::parse_env_output(&stdout)?;

    assert_eq!(None, sudo_env.get(env_name));

    Ok(())
}

// also see the `if_value_starts_with_parentheses_variable_is_removed` test
fn key_value_where_value_is_parentheses_glob(env_list: EnvList) -> Result<()> {
    let env_name = "SHOULD_BE_PRESERVED";
    let env_val = "() 42";
    let env = Env([
        SUDOERS_ALL_ALL_NOPASSWD,
        &format!("Defaults {env_list} = \"{env_name}=()*\""),
    ])
    .build()?;

    let stdout = Command::new("env")
        .arg(format!("{env_name}={env_val}"))
        .args(["sudo", "env"])
        .exec(&env)?
        .stdout()?;
    let sudo_env = helpers::parse_env_output(&stdout)?;

    assert_eq!(Some(env_val), sudo_env.get(env_name).copied());

    Ok(())
}

fn minus_equal_removes(env_list: EnvList) -> Result<()> {
    let env_name1 = "SHOULD_BE_PRESERVED";
    let env_val1 = "42";
    let env_name2 = "SHOULD_BE_REMOVED";
    let env_val2 = "24";
    let env = Env([
        SUDOERS_ALL_ALL_NOPASSWD,
        &format!("Defaults {env_list} = \"{env_name1} {env_name2}\""),
        &format!("Defaults {env_list} -= {env_name2}"),
    ])
    .build()?;

    let stdout = Command::new("env")
        .arg(format!("{env_name1}={env_val1}"))
        .arg(format!("{env_name2}={env_val2}"))
        .args(["sudo", "env"])
        .exec(&env)?
        .stdout()?;
    let sudo_env = helpers::parse_env_output(&stdout)?;

    assert_eq!(Some(env_val1), sudo_env.get(env_name1).copied());
    assert!(sudo_env.get(env_name2).is_none());

    Ok(())
}

fn minus_equal_an_element_not_in_the_list_is_not_an_error(env_list: EnvList) -> Result<()> {
    let env_name = "SHOULD_BE_REMOVED";
    let env_val = "42";
    let env = Env([
        SUDOERS_ALL_ALL_NOPASSWD,
        &format!("Defaults {env_list} -= {env_name}"),
    ])
    .build()?;

    let output = Command::new("env")
        .arg(format!("{env_name}={env_val}"))
        .args(["sudo", "env"])
        .exec(&env)?;

    // no diagnostics in this case
    assert!(output.stderr().is_empty());

    let stdout = output.stdout()?;
    let sudo_env = helpers::parse_env_output(&stdout)?;

    assert!(sudo_env.get(env_name).is_none());

    Ok(())
}

fn bang_clears_the_whole_list(env_list: EnvList) -> Result<()> {
    let env_name1 = "SHOULD_BE_REMOVED";
    let env_name2 = "ALSO_SHOULD_BE_REMOVED";
    let env_val1 = "42";
    let env_val2 = "24";
    let env = Env([
        SUDOERS_ALL_ALL_NOPASSWD,
        &format!("Defaults {env_list} = \"{env_name1} {env_name2}\""),
        &format!("Defaults !{env_list}"),
    ])
    .build()?;

    let stdout = Command::new("env")
        .arg(format!("{env_name1}={env_val1}"))
        .arg(format!("{env_name2}={env_val2}"))
        .args(["sudo", "env"])
        .exec(&env)?
        .stdout()?;

    let sudo_env = helpers::parse_env_output(&stdout)?;

    assert!(sudo_env.get(env_name1).is_none());
    assert!(sudo_env.get(env_name1).is_none());

    Ok(())
}

fn can_append_after_bang(env_list: EnvList) -> Result<()> {
    let env_name1 = "SHOULD_BE_REMOVED";
    let env_name2 = "SHOULD_BE_PRESERVED";
    let env_val1 = "42";
    let env_val2 = "24";
    let env = Env([
        SUDOERS_ALL_ALL_NOPASSWD,
        &format!("Defaults {env_list} = {env_name1}"),
        &format!("Defaults !{env_list}"),
        &format!("Defaults {env_list} += {env_name2}"),
    ])
    .build()?;

    let stdout = Command::new("env")
        .arg(format!("{env_name1}={env_val1}"))
        .arg(format!("{env_name2}={env_val2}"))
        .args(["sudo", "env"])
        .exec(&env)?
        .stdout()?;

    let sudo_env = helpers::parse_env_output(&stdout)?;

    assert!(sudo_env.get(env_name1).is_none());
    assert_eq!(Some(env_val2), sudo_env.get(env_name2).copied());

    Ok(())
}

fn can_override_after_bang(env_list: EnvList) -> Result<()> {
    let env_name1 = "SHOULD_BE_REMOVED";
    let env_name2 = "SHOULD_BE_PRESERVED";
    let env_val1 = "42";
    let env_val2 = "24";
    let env = Env([
        SUDOERS_ALL_ALL_NOPASSWD,
        &format!("Defaults {env_list} = {env_name1}"),
        &format!("Defaults !{env_list}"),
        &format!("Defaults {env_list} = {env_name2}"),
    ])
    .build()?;

    let stdout = Command::new("env")
        .arg(format!("{env_name1}={env_val1}"))
        .arg(format!("{env_name2}={env_val2}"))
        .args(["sudo", "env"])
        .exec(&env)?
        .stdout()?;

    let sudo_env = helpers::parse_env_output(&stdout)?;

    assert!(sudo_env.get(env_name1).is_none());
    assert_eq!(Some(env_val2), sudo_env.get(env_name2).copied());

    Ok(())
}

// DISPLAY, PATH and TERM are env vars preserved by sudo by default
// they appear to be part of the default `env_keep` list
fn equal_can_disable_preservation_of_vars_display_path_but_not_term(
    env_list: EnvList,
) -> Result<()> {
    let env = Env([
        SUDOERS_ALL_ALL_NOPASSWD,
        &format!("Defaults {env_list} = WHATEVER"),
    ])
    .build()?;

    let sudo_abs_path = Command::new("which").arg("sudo").exec(&env)?.stdout()?;
    let env_abs_path = Command::new("which").arg("env").exec(&env)?.stdout()?;

    let term = "some-term";
    let stdout = Command::new("env")
        .arg("PATH=some-path")
        .arg("DISPLAY=some-display")
        .arg(format!("TERM={term}"))
        .args([sudo_abs_path, env_abs_path])
        .exec(&env)?
        .stdout()?;

    let sudo_env = helpers::parse_env_output(&stdout)?;

    // can be disabled
    assert!(sudo_env.get("DISPLAY").is_none());
    assert_eq!(Some(SUDO_ENV_DEFAULT_PATH), sudo_env.get("PATH").copied());

    // cannot be disabled
    assert_eq!(Some(term), sudo_env.get("TERM").copied());

    Ok(())
}

fn equal_minus_can_disable_preservation_of_vars_display_path_but_not_term(
    env_list: EnvList,
) -> Result<()> {
    let env = Env([
        SUDOERS_ALL_ALL_NOPASSWD,
        &format!("Defaults {env_list} -= \"DISPLAY PATH TERM\""),
    ])
    .build()?;

    let sudo_abs_path = Command::new("which").arg("sudo").exec(&env)?.stdout()?;
    let env_abs_path = Command::new("which").arg("env").exec(&env)?.stdout()?;

    let term = "some-term";
    let stdout = Command::new("env")
        .arg("PATH=some-path")
        .arg("DISPLAY=some-display")
        .arg(format!("TERM={term}"))
        .args([sudo_abs_path, env_abs_path])
        .exec(&env)?
        .stdout()?;

    let sudo_env = helpers::parse_env_output(&stdout)?;

    // can be disabled
    assert!(sudo_env.get("DISPLAY").is_none());
    assert_eq!(Some(SUDO_ENV_DEFAULT_PATH), sudo_env.get("PATH").copied());

    // cannot be disabled
    assert_eq!(Some(term), sudo_env.get("TERM").copied());

    Ok(())
}

fn bang_can_disable_preservation_of_vars_display_path_but_not_term(
    env_list: EnvList,
) -> Result<()> {
    let env = Env([SUDOERS_ALL_ALL_NOPASSWD, &format!("Defaults !{env_list}")]).build()?;

    let sudo_abs_path = Command::new("which").arg("sudo").exec(&env)?.stdout()?;
    let env_abs_path = Command::new("which").arg("env").exec(&env)?.stdout()?;

    let term = "some-term";
    let stdout = Command::new("env")
        .arg("PATH=some-path")
        .arg("DISPLAY=some-display")
        .arg(format!("TERM={term}"))
        .args([sudo_abs_path, env_abs_path])
        .exec(&env)?
        .stdout()?;

    let sudo_env = helpers::parse_env_output(&stdout)?;

    // can be disabled
    assert!(sudo_env.get("DISPLAY").is_none());
    assert_eq!(Some(SUDO_ENV_DEFAULT_PATH), sudo_env.get("PATH").copied());

    // cannot be disabled
    assert_eq!(Some(term), sudo_env.get("TERM").copied());

    Ok(())
}
