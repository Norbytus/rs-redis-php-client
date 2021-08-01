pub (crate) fn arg_to_string(arg: ext_php_rs::php::args::Arg, arg_name: &str, data: &mut String) {
    if let Some(arg) = arg.val::<String>() {
        data.push_str(&arg)
    } else {
        ext_php_rs::php::exceptions::throw(
            ext_php_rs::php::class::ClassEntry::type_error(),
            format!("Wrong {} argument", arg_name).as_str()
        );

        return;
    }
}
