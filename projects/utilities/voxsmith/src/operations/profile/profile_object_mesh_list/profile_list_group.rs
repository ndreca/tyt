/// The profiles one origin supplies, an entry of a
/// [`profile_list`](crate::operations::profile::profile_list()).
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ProfileListGroup {
    /// The origin supplying the profiles, a config path or `built in`.
    pub origin: String,

    /// The profile names, in name order.
    pub profiles: Vec<String>,
}
