from setuptools import setup
from setuptools.dist import Distribution


class BinaryDistribution(Distribution):
    def has_ext_modules(self):
        return True


try:
    from wheel.bdist_wheel import bdist_wheel as _bdist_wheel
except Exception:  # pragma: no cover - wheel not installed
    cmdclass = {}
else:
    class bdist_wheel(_bdist_wheel):
        def finalize_options(self):  # pragma: no cover - distutils glue
            super().finalize_options()
            self.root_is_pure = False

    cmdclass = {"bdist_wheel": bdist_wheel}


setup(distclass=BinaryDistribution, cmdclass=cmdclass)
