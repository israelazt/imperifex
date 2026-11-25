//! Reading the name of a GPU engine counter instance, such as
//! `pid_15140_luid_0x00000000_0x00014AB0_phys_0_eng_0_engtype_3D` (the form
//! seen on the reference machine). Plain code, tested everywhere.

/// The process of an instance of a 3D engine; `None` for any other engine or a
/// name of another form.
pub(crate) fn pid_of_3d_engine(instance: &str) -> Option<u32> {
    let (pid, rest) = instance.strip_prefix("pid_")?.split_once('_')?;
    rest.ends_with("_engtype_3D").then(|| pid.parse().ok())?
}

#[cfg(test)]
mod tests {
    use super::*;

    const SEEN: &str = "pid_15140_luid_0x00000000_0x00014AB0_phys_0_eng_0_engtype_3D";

    #[test]
    fn the_process_of_a_3d_engine_is_read_from_its_name() {
        assert_eq!(pid_of_3d_engine(SEEN), Some(15140));
    }

    #[test]
    fn other_engines_and_other_names_are_nobodys() {
        for name in [
            "pid_15140_luid_0x0_0x1_phys_0_eng_3_engtype_Copy",
            "pid_x_luid_0x0_0x1_phys_0_eng_0_engtype_3D",
            "luid_0x0_0x1_phys_0_eng_0_engtype_3D",
            "pid_",
            "",
        ] {
            assert_eq!(pid_of_3d_engine(name), None, "{name}");
        }
    }
}
