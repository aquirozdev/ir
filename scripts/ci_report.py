"""Record real checks and provenance. This is a regression report, not an ML benchmark."""
import json
import platform
import re
import subprocess
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]

def command(*args):
    return subprocess.check_output(args, cwd=ROOT, text=True).strip()


def main():
    report = json.loads((ROOT / 'results/demo.json').read_text())
    checks = report['checks']
    if not checks or not all(check['passed'] for check in checks):
        raise AssertionError('demo checks did not pass')
    log = (ROOT / 'results/tests.log').read_text()
    counts = re.findall(r'test result: ok\. (\d+) passed; (\d+) failed;', log)
    if not counts or any(int(failed) for _, failed in counts):
        raise AssertionError('test result missing or contains failures')
    passed = sum(int(passed) for passed, _ in counts)
    if passed == 0:
        raise AssertionError('no regression tests ran')
    result = {
        'kind': 'runtime-regression', 'sir_version': report['sir_version'],
        'tests_passed': passed, 'demo_checks_passed': len(checks),
        'commit': command('git', 'rev-parse', 'HEAD'),
        'worktree_dirty': bool(command('git', 'status', '--porcelain')),
        'rustc': command('rustc', '--version'),
        'cargo': command('cargo', '--version'),
        'platform': platform.platform(),
        'inference_tested': False,
    }
    (ROOT / 'results/ci-report.json').write_text(json.dumps(result, indent=2) + '\n')
    print(json.dumps(result, indent=2))


if __name__ == '__main__':
    main()
