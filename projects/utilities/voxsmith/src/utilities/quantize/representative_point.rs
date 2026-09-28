use crate::utilities::QuantizePoint;

/// A cluster's representative: its most-sampled point, ties to the lowest
/// material id.
pub(crate) fn representative_point(cluster: &[QuantizePoint]) -> QuantizePoint {
    cluster
        .iter()
        .copied()
        .max_by(|a, b| {
            a.population
                .cmp(&b.population)
                .then_with(|| b.material_id.to_u32().cmp(&a.material_id.to_u32()))
        })
        .expect("a cluster holds at least one point")
}
