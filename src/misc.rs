use std::error::Error;

pub fn download_text(url: &str) -> Result<String, Box<dyn Error>> {
    let dur = std::time::Duration::from_secs(5);

    let agent: ureq::Agent = ureq::AgentBuilder::new()
        .timeout_read(dur)
        .timeout_write(dur)
        .build();

    let text = agent.get(url).call()?.into_string()?;
    Ok(text)
}

#[macro_export]
macro_rules! fn_name {
    () => {{
        fn f() {}
        fn type_name_of<T>(_: T) -> &'static str {
            std::any::type_name::<T>()
        }
        let name = type_name_of(f);

        $crate::misc::up_to_last_two_path_elements(&name[..name.len() - 3])
    }};
}

pub fn up_to_last_two_path_elements(name: &'static str) -> &'static str {
    let patt = "::";
    let offset = patt.len();
    let mut i = name.rfind(patt);
    match i {
        Some(i) => match name[..i].rfind(patt) {
            Some(i) => &name[i + offset..],
            None => &name[i + offset..],
        },
        None => name,
    }
}
