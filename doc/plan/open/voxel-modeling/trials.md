# Trials

Step S12 of the [checklist](checklist.md) runs these prompts through the
voxel-modeling skill.

## Running a trial

Each trial runs in a fresh Claude Code session in its own directory outside the
repository, such as `~/voxel-trials/chair`. A session inside the repository
could read the builder and the docs in place of the skill. The directory holds
the `SKILL.md` that the installed vxl printed, as the vxl
[README](../../../../projects/utilities/vxl/README.md#models) shows, and a
`.claude/settings.json` that allows `vxl` commands and edits.

The prompt goes in as a user would write it, with no mention of the skill. The
trial then also tests whether the skill's description loads the skill. The
session runs passes until Claude calls the model done, and the person steps in
only where a user would.

Each trial adds a section below, titled with the prompt's number. The section
records the passes the trial took, the failures, the operations and report data
Claude wanted and lacked, and where the model's colors read flat.

## Prompts

1. make a voxel chair with oak and ornate jewels in the top
2. make a voxel lantern with a candle glowing inside
3. make a voxel treasure chest with the lid open and gold coins inside
4. make a voxel sword with a gold hilt and a gem in the pommel
5. make a voxel oak tree
6. make a voxel stone well with a wooden roof and a bucket on a rope
7. make a voxel robot whose head and arms can turn
8. make a voxel mushroom cottage with a round door and windows
9. make a voxel glass potion bottle with a cork and red potion inside
10. make a voxel wooden cart with four wheels
