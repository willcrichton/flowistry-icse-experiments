This is the anonymized supplementary materials for ICSE 2023 submission #389, "Helping Programmers Find Relevant Code with Modular Slices".

The directory structure is:
* crates/flowistry - source code for Focus Mode
  * crates/flowistry - Rust source code for Flowistry [Crichton et al. 2022]
  * crates/flowistry_ide - Rust source code for Focus Mode program slicer (Section 2)
  * ide - Typescript source code for Focus Mode VSCode integration (Section 4)
* crates/eval - source code for dataset analysis (Section 3)
* crates/user-study - source code for user study (Section 5)
  * src/tutorial.rs - Focus Mode tutorial
  * src/progress.rs - Warm-up task
  * src/url.rs, src/countries.rs, src/cli.rs - Url, Median, and Command-Line tasks
