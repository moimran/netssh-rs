#!/usr/bin/env python3
"""
Setup script for netssh_rs Python bindings.
This is an alternative to using maturin directly.
"""

from setuptools import setup
from setuptools_rust import Binding, RustExtension

setup(
    name="netssh_rs",
    version="0.1.0",
    description="Python bindings for netssh-rs - SSH connection handler for network devices",
    long_description=open("python/README.md").read(),
    long_description_content_type="text/markdown",
    author="Your Name",
    author_email="your.email@example.com",
    url="https://github.com/yourusername/netssh-rs",
    classifiers=[
        "Programming Language :: Python :: 3",
        "Programming Language :: Python :: 3.7",
        "Programming Language :: Python :: 3.8",
        "Programming Language :: Python :: 3.9",
        "Programming Language :: Python :: 3.10",
        "Programming Language :: Python :: 3.11",
        "Programming Language :: Rust",
        "Topic :: System :: Networking",
        "Topic :: Software Development :: Libraries :: Python Modules",
        "License :: OSI Approved :: MIT License",
    ],
    python_requires=">=3.7",
    rust_extensions=[
        RustExtension(
            "netssh_rs",
            binding=Binding.PyO3,
            features=["pyo3/extension-module"],
        )
    ],
    packages=["netssh_rs"],
    package_dir={"netssh_rs": "python"},
    include_package_data=True,
    zip_safe=False,
) 