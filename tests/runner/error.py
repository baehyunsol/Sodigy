import re
from typing import Tuple

class Error:
    level: str  # error | warning
    index: int
    title: str
    body: str

    def __init__(self, level, index, title, body):
        self.level = level
        self.index = index
        self.title = title
        self.body = body

def parse_errors(stderr: str) -> Tuple[list[Error], list[Error]]:  # (errors, warnings)
    result = []
    level, index, title, body = None, None, None, []

    for line in stderr.split("\n"):
        if (r := re.search(r"^error\s\(e\-(\d{4})\)\:(.+)", line)) is not None:
            result.append(Error(level, index, title, "\n".join(body)))
            level = "error"
            index, title = r.groups()
            index = int(index)
            body = []

        elif (r := re.search(r"^warning\s\(w\-(\d{4})\)\:(.+)", line)) is not None:
            result.append(Error(level, index, title, "\n".join(body)))
            level = "warning"
            index, title = r.groups()
            index = int(index)
            body = []

        elif (r := re.search("^Finished\:\s(\d+)\serror(?:s)?\sand\s(\d+)\swarning(?:s)?", line)) is not None:
            result.append(Error(level, index, title, "\n".join(body)))
            error_count = int(r.group(1))
            warning_count = int(r.group(2))

        elif level is not None:
            body.append(line)

    errors = [e for e in result if e.level == "error"]
    warnings = [w for w in result if w.level == "warning"]

    assert len(errors) == error_count
    assert len(warnings) == warning_count
    return errors, warnings
