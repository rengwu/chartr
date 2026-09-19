#!/usr/bin/env python3
"""Download and verify the AUR recipe attached to a published stable release."""
import hashlib
import json
from pathlib import Path
import re
import subprocess
import sys
import tarfile
import tempfile

REPOSITORY = 'rengwu/chartr'


def stable_version(tag):
    if not re.fullmatch(r'v[0-9]+\.[0-9]+\.[0-9]+', tag):
        raise ValueError('AUR publication requires a stable vMAJOR.MINOR.PATCH tag')
    return tag[1:]


def validate_release(tag, release):
    stable_version(tag)
    if release['tagName'] != tag or release['isDraft'] or release['isPrerelease']:
        raise ValueError('AUR publication requires a published stable release')


def prepare_recipe(tag, bundle, manifest, output):
    version = stable_version(tag)
    checksums = {}
    for line in manifest.read_text().splitlines():
        digest, name = line.split()
        if not re.fullmatch(r'[0-9a-f]{64}', digest) or name in checksums:
            raise ValueError('invalid or duplicate release checksum')
        checksums[name] = digest
    if hashlib.sha256(bundle.read_bytes()).hexdigest() != checksums[bundle.name]:
        raise ValueError('AUR recipe checksum does not match the release')
    with tarfile.open(bundle, 'r:gz') as archive:
        members = archive.getmembers()
        if (sorted(m.name for m in members) != ['.SRCINFO', 'PKGBUILD']
                or any(not m.isfile() or m.size > 65536 for m in members)):
            raise ValueError('AUR bundle must contain only PKGBUILD and .SRCINFO files')
        files = {m.name: archive.extractfile(m).read() for m in members}
    metadata = {}
    for line in files['.SRCINFO'].decode().splitlines():
        if ' = ' in line:
            key, value = line.strip().split(' = ', 1)
            metadata.setdefault(key, []).append(value)
    source = f'chartr-{version}-linux-x86_64.tar.gz'
    expected = {
        'pkgbase': ['chartr-bin'], 'pkgname': ['chartr-bin'],
        'pkgver': [version], 'pkgrel': ['1'], 'arch': ['x86_64'],
        'source': [f'https://github.com/{REPOSITORY}/releases/download/{tag}/{source}'],
        'sha256sums': [checksums[source]],
        'provides': [f'chartr={version}'], 'conflicts': ['chartr'],
    }
    for key, value in expected.items():
        if metadata.get(key) != value:
            raise ValueError(f'AUR metadata {key} does not match the release')
    output.mkdir(parents=True, exist_ok=True)
    for name, contents in files.items():
        (output / name).write_bytes(contents)


def main():
    if len(sys.argv) != 3:
        raise ValueError('usage: prepare-aur-release.py <release-tag> <recipe-directory>')
    tag, destination = sys.argv[1:]
    version = stable_version(tag)
    release = json.loads(subprocess.check_output([
        'gh', 'release', 'view', tag, '--repo', REPOSITORY,
        '--json', 'tagName,isDraft,isPrerelease'], text=True))
    validate_release(tag, release)
    with tempfile.TemporaryDirectory() as directory:
        work = Path(directory)
        bundle = work / f'chartr-{version}-aur.tar.gz'
        manifest = work / 'SHA256SUMS-x86_64'
        subprocess.run(['gh', 'release', 'download', tag, '--repo', REPOSITORY,
                        '--pattern', bundle.name, '--pattern', manifest.name,
                        '--dir', str(work)], check=True)
        prepare_recipe(tag, bundle, manifest, Path(destination))


if __name__ == '__main__':
    try:
        main()
    except (ValueError, KeyError, OSError, tarfile.TarError, subprocess.CalledProcessError) as error:
        sys.exit(str(error))
