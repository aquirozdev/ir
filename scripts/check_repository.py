"""Check fixtures against this draft's schema subset; not a general schema engine."""
import json
import re
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
SCHEMA = json.loads((ROOT / 'schemas/sir-v0.schema.json').read_text())


def check(value, spec):
    if '$ref' in spec:
        target = SCHEMA
        for part in spec['$ref'].removeprefix('#/').split('/'):
            target = target[part]
        return check(value, target)
    if 'oneOf' in spec:
        matches = 0
        for choice in spec['oneOf']:
            try:
                check(value, choice)
                matches += 1
            except ValueError:
                pass
        if matches != 1:
            raise ValueError('oneOf must match exactly once')
        return
    if 'const' in spec and value != spec['const']:
        raise ValueError('incorrect constant')
    kind = spec.get('type')
    if kind == 'object':
        if not isinstance(value, dict):
            raise ValueError('expected object')
        if set(spec['required']) - value.keys() or value.keys() - spec['properties'].keys():
            raise ValueError('missing or unknown fields')
        for key, item in value.items():
            check(item, spec['properties'][key])
    elif kind == 'array':
        if not isinstance(value, list) or len(value) < spec.get('minItems', 0) or len(value) > spec.get('maxItems', 10**9):
            raise ValueError('invalid array')
        for item in value:
            check(item, spec['items'])
    elif kind == 'string':
        if not isinstance(value, str) or len(value) > spec.get('maxLength', 10**9):
            raise ValueError('invalid string')
        if 'pattern' in spec and not re.fullmatch(spec['pattern'], value):
            raise ValueError('invalid identifier')
    elif kind == 'integer':
        if type(value) is not int or not spec.get('minimum', -2**63) <= value <= spec.get('maximum', 2**63-1):
            raise ValueError('invalid integer')
    elif kind == 'boolean':
        if type(value) is not bool:
            raise ValueError('invalid boolean')


def main():
    fixture = json.loads((ROOT / 'examples/expenses.entities.json').read_text())
    check(fixture, SCHEMA)
    program = json.loads((ROOT / 'examples/expenses.sir.json').read_text())
    check(program, SCHEMA)
    for altered in [dict(fixture, arbitrary=[]), dict(fixture, entities=[]),
                    dict(fixture, name='invalid name'), dict(fixture, sir_version='future')]:
        try:
            check(altered, SCHEMA)
        except ValueError:
            continue
        raise AssertionError('invalid fixture accepted')
    for path in ROOT.rglob('*.md'):
        for link in re.findall(r'\]\(([^)]+)\)', path.read_text()):
            if '://' not in link and not link.startswith('#'):
                if not (path.parent / link.split('#')[0]).exists():
                    raise AssertionError(f'broken link: {path}: {link}')
    print('Fixture contract, negative fixtures and documentation links passed.')


if __name__ == '__main__':
    main()
