from setuptools import setup

setup(
    name="netssh_rs",
    version="0.1.0",
    packages=["netssh_rs"],
    package_data={
        "netssh_rs": ["py.typed", "stubs/*.pyi"],
    },
) 