from error import Error

class RunResult:
    # success | test-fail | compile-fail | misc-error | timeout
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

        except Exception as e:
            raise Exception(f"error in the expectation: {e}")

        expect = names["expect"]
        expect(self)

def parse_expectation(code: str) -> str:
    expectation = None

    for line in code.split("\n"):
        normalized_line = "".join([c for c in line if c != " "]).lower()

        if normalized_line.startswith("/*<expect>"):
            expectation = ""
            continue

        elif normalized_line.endswith("</expect>*/"):
            break

        if isinstance(expectation, str):
            expectation += "\n"
            expectation += line

    if expectation is None:
        expectation = """
def expect(result):
    if result.status != "success":
        raise Exception(result.status)
"""

    return expectation
