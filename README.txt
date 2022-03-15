This is the PLDI 2022 research artifact submission for "Modular Information Flow Through Ownership" (Crichton et al.).


# Getting Started Guide

This artifact is entirely contained within a Docker image. The Docker image was built with Docker version 18.09.2. Enter the image by running:

```bash
docker load -i flowistry.tar
docker run -p 8888:8888 -ti flowistry bash
```

> Make sure you include `-p 8888:8888` so you can visit websites run from the artifact in your browser.

All artifact files are in the `/flowistry` directory. The subdirectories:
* `crates/` contains all code:
  * `crates/flowistry/` is the core package containing the analysis.
  * `crates/eval/` is the harness for running the evaluation.
* `data/` contains the inputs and outputs for the evaluation:
  * `data/repos/` are the repositories analyzed.
  * `data/*.json` are the outputs of the evaluation for a given repository.
* `notebooks/` contains the Jupyter notebooks used to run and analyze the evaluation:
  * `notebooks/execute.ipynb` runs the evaluation.
  * `notebooks/analysis.ipynb` generates charts and statistics used in the paper.

You can verify that the codebase works correctly by running the unit tests:

```bash
cd /flowistry/crates/flowistry
cargo test
```


# Step-by-Step Instructions


## Evaluation

To reproduce the evaluation (Section 5), first you need to launch Jupyter and open the `execute.ipynb` notebook from within the container.

```
cd /flowistry
jupyter notebook --allow-root --port 8888 --ip 0.0.0.0
```

Then open http://localhost:8888/notebooks/notebooks/execute.ipynb in your browser. You will need to provide a password -- you can find it in the `jupyter` CLI output. To run the evaluation, click "Cell > Run All".

**NOTE:** this evaluation may take up to 6 hours to run. If you just want to reproduce the statistics/charts, we have included the generated data (`data/*.json`) in the Docker image. If you want to reproduce the generated data, then you will need to delete those files (`rm data/*.json`) and then run the notebook. If you want to just reproduce a single repository, you can delete only one file (e.g. `rm data/rayon.json`) and run the notebook.

Once the evaluation is complete, then you can visit http://localhost:8888/notebooks/notebooks/analysis.ipynb for the charts and statistics. Again click "Cell > Run All". Then you can read through the notebook to compare the figures and numbers in the paper against the analysis. Each cell is annotated with which part of the paper it corresponds to. Specifically, you can find:

* The dataset summary in Table 1
* The slice size comparisons in Figures 2, 3, and 4
* The statistics mentioned in 5.2 and 5.4

An important note on reproducibility: one mistake we made in the original evaluation is that while we pinned our repositories to specific commits, we did not also pin to specific *lock files* (Cargo.lock). So several months later, when we tried to rerun the evaluation, upstream changes to dependencies caused compilation errors in the crates in our dataset. To fix this, we had to update the version of the Rust compiler used in our tool. The provided Docker image should now be indefinitely stable (including lockfiles). However, the newer compiler (nightly-2022-02-17) emits different instructions than the older one (nightly-2021-10-08), so this affects some of our quantitative results around the number of slices per crate and the size of slices.

The dataset summary should be exactly the same except the # Vars column. Figures 2, 3, and 4 and the statistics in 5.2 and 5.4 will be slightly different, but the numbers have not changed in a way that affects the conclusion of the paper (which also supports our hypothesis that this effect is stable!). For example, the % of cases where the modular condition is different from the whole-program condition changed from 10% to 6%. These changes will appear in the camera-ready, but AE requires us to only provide the submission version of the paper.


## Documentation

We have designed our tool to be easy to use and extend. The documentation for the IDE extension is described in `crates/flowistry/README.md`. The documentation for the codebase itself can be viewed by running:

```bash
cd /flowistry/crates/flowistry/target/doc/
python3 -m http.server 8888
```

Then visit http://localhost:8888/flowistry/ in your browser.
