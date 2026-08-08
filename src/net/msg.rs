pub const MSG_SCAN: &str = r#"
{
  "msg": {
    "cmd": "scan",
    "data": {
      "accout_topic": "reserve"
    }
  }
}
"#;

pub const MSG_STATUS: &str = r#"
{
    "msg": {
        "cmd": "devStatus",
        "data": {}
    }
}
"#;
