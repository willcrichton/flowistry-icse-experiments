This is the anonymized supplementary materials for OOPSLA 2022 submission #229, "Helping Programmers Find Relevant Code with Modular Slices".

The directory structure is:
* crates/flowistry - source code for Focus Mode (Section 2)
  * crates/flowistry - Rust source code for Flowistry [Crichton et al. 2022]
  * crates/flowistry_ide - Rust source code for Focus Mode program slicer
  * ide - Typescript source code for FocusMode VSCode integration
* crates/user-study - source code for user study (Section 3)
  * src/tutorial.rs - Focus Mode tutorial
  * src/progress.rs - Warm-up task
  * src/url.rs, src/countries.rs, src/cli.rs - Url, Median, and Command-Line tasks
* crates/eval - source code for dataset analysis (Section 4)
