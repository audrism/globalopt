# PyPI Release Guide

This package is built with maturin and publishes a source distribution plus wheel(s).

## 1) Prerequisites

- Python 3.9+
- Rust toolchain (`cargo`, `rustc`)
- Build tools:

```bash
python3 -m pip install --upgrade pip
python3 -m pip install maturin build twine pytest
```

## 2) Run tests

From repository root:

```bash
PYTHONPATH=python pytest -q python/tests
```

## 3) Build distributions

From repository root:

```bash
cd python
python3 -m maturin build --release --out dist
python3 -m build --sdist --outdir dist
```

Artifacts will be under `python/dist/`.

## 4) Validate artifacts

```bash
python3 -m twine check dist/*
```

## 5) Upload to TestPyPI (recommended first)

Create an API token at TestPyPI and use it as the password.

```bash
python3 -m twine upload --repository testpypi dist/*
```

## 6) Upload to PyPI

Create a PyPI API token and upload:

```bash
python3 -m twine upload dist/*
```

## 7) Verify install

```bash
python3 -m pip install globalopt-py
python3 -c "import globalopt; print('ok')"
```
