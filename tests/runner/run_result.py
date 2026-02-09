from error import Error

class RunResult:
    # success | test-error | compile-error | misc-error | timeout
    # "misc-error" is when the compilation failed, but due to a bug in the compiler.
    status: str
    errors: list[Error]
    warnings: list[Error]

    def __init__(self, status, errors, warnings):
        self.status = status
        self.errors = errors
        self.warnings = warnings

    def expect(self, expectation):
        names = {}

        try:
            exec(expectation, names)
            expect = names["expect"]

        except Exception as e:
            raise Exception(f"error in the expectation: {e}")

        # TODO: collect (Sodigy-runtime) test errors
        expect(
            status=self.status,
            errors=self.errors,
            warnings=self.warnings,
            success=self.status == "success",
            test_error=self.status == "test-error",
            compile_error=self.status == "compile-error",
            misc_error=self.status == "misc-error",
            timeout=self.status == "timeout",
        )

def parse_expectation(code: str) -> str:
    expectation = None

    for line in code.split("\n"):
        if line.startswith("//#"):
            line = line[3:]

            if isinstance(expectation, str):
                expectation += f"\n    {line.strip()}"

            else:
                expectation = f"""
def expect(status, errors, warnings, success, test_error, compile_error, misc_error, timeout):
    import re
    {line.strip()}
"""

    if expectation is None:
        expectation = """
def expect(status, **kwargs):
    if status != "success":
        raise Exception(status)
"""

    return expectation
