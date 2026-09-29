use crate::{
    Error, Result,
    operations::object::{
        CheckedDestination, Destination, MeshElement, MeshEnvironment, MeshGeometry, MeshRecord,
        Swatches,
    },
};
use vox_value_language::{CheckedProgram, EvaluatedProgram, check, eval, parse};
use voxcore::VoxObject;

/// The program run over one geometry, with every destination checked in
/// its end scope.
pub struct ProgramRun {
    pub checked: CheckedProgram,

    pub evaluated: EvaluatedProgram,

    pub destinations: Vec<CheckedDestination>,
}

impl ProgramRun {
    /// Runs `record` over `geometry`.
    pub(crate) fn over(
        object: &VoxObject,
        swatches: &Swatches<'_>,
        record: &MeshRecord,
        geometry: &MeshGeometry,
    ) -> Result<Self> {
        let environment =
            MeshEnvironment::bind(object, swatches, geometry, &record.computed_bindings)?;

        let (checked, evaluated) = run_program(&record.program, &environment)?;

        let destinations = Destination::of_record(record)?
            .into_iter()
            .map(|destination| destination.check(&checked))
            .collect::<Result<_>>()?;

        Ok(ProgramRun {
            checked,
            evaluated,
            destinations,
        })
    }
}

/// Parses, checks, and evaluates `program` over `environment`, every error
/// rising from the program element.
fn run_program(
    program: &str,
    environment: &MeshEnvironment,
) -> Result<(CheckedProgram, EvaluatedProgram)> {
    let parsed = parse(program).map_err(|error| Error::mesh_record(MeshElement::Program, error))?;

    let checked = check(parsed, &environment.types)
        .map_err(|error| Error::mesh_record(MeshElement::Program, error))?;

    let evaluated = eval(&checked, &environment.values)
        .map_err(|error| Error::mesh_record(MeshElement::Program, error))?;

    Ok((checked, evaluated))
}
