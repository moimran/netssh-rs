#!/usr/bin/env python
"""
Setup script for netssh_rs Python bindings.
This is an alternative to using maturin directly.
"""

from setuptools import setup, find_packages
from setuptools_rust import Binding, RustExtension
import os

# Read the contents of README.md
this_directory = os.path.abspath(os.path.dirname(__file__))
with open(os.path.join(this_directory, "README.md"), encoding="utf-8") as f:
    long_description = f.read()

setup(
    name="netssh-rs",
    version="0.1.0",
    description="Python bindings for netssh-rs Rust library for network automation",
    long_description=long_description,
    long_description_content_type="text/markdown",
    author="Imran",
    author_email="your.email@example.com",
    url="https://github.com/yourusername/netssh-rs",
    classifiers=[
        "Development Status :: 3 - Alpha",
        "Intended Audience :: Developers",
        "Programming Language :: Python :: 3",
        "Programming Language :: Python :: 3.6",
        "Programming Language :: Python :: 3.7",
        "Programming Language :: Python :: 3.8",
        "Programming Language :: Python :: 3.9",
        "Programming Language :: Python :: 3.10",
        "Programming Language :: Rust",
        "Topic :: Software Development :: Libraries",
        "Topic :: System :: Networking",
        "Typing :: Typed",
        "License :: OSI Approved :: MIT License",
    ],
    python_requires=">=3.6",
    rust_extensions=[
        RustExtension(
            "netssh_rs",
            binding=Binding.PyO3,
            features=["pyo3/extension-module"],
        )
    ],
    packages=find_packages(),
    package_dir={
        "netssh_rs": "python",
        "textfsm": "textfsm"
    },
    package_data={
        "netssh_rs": ["py.typed", "*.pyi"],
        "textfsm": ["py.typed", "*.pyi", "templates/*"],
    },
    install_requires=[
        "textfsm>=1.1.0",
    ],
    include_package_data=True,
    zip_safe=False,  # Required for mypy to find the type information
) 