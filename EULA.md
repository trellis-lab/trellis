# Trellis End-User License Agreement (EULA)

**Last updated: 2026**

This End-User License Agreement ("Agreement") is a legal agreement between you
(an individual or an entity, "You") and **Zoran Vukoszavlyev (TrellisLab)**
("Licensor") for the Trellis software product.

This Agreement governs the **Trellis CLI binary, the Trellis Docker images, and
all related commercial features** (collectively, the "Software"). It does **not**
govern the source files in this repository, which are licensed separately under
the [MIT License](LICENSE).

By downloading, installing, or using the Software, You agree to be bound by this
Agreement. If You do not agree, do not install or use the Software.

---

## 1. Nature of the Software

The Software is **proprietary and closed-source**. It is licensed, not sold. The
Licensor retains all right, title, and interest in and to the Software, including
all intellectual property rights. No source code is provided.

## 2. License grant

Subject to this Agreement, the Licensor grants You a non-exclusive,
non-transferable, revocable license to install and use the Software as follows:

- **Free tier.** You may use the free features at no charge: rendering single
  diagrams to PNG (`trellis render`), syntax validation (`trellis validate`),
  the VS Code extension live preview and PNG export, and PNG output via the MCP
  server.
- **Commercial features.** Use of `render-batch` and of SVG, HTML, and Draw.io
  output (via the CLI or the MCP server) requires a valid, paid license key.
  Each activated device consumes one seat from Your subscription. See
  [docs/licensing.md](docs/licensing.md) for activation and seat details.

## 3. Restrictions

You may **not**:

- reverse-engineer, decompile, or disassemble the Software, except to the extent
  this restriction is prohibited by applicable law;
- circumvent, disable, or tamper with the license-key, seat, or feature-gating
  mechanisms;
- redistribute, resell, sublicense, rent, or lease the Software or license keys
  without the Licensor's prior written consent;
- remove or alter any proprietary notices.

## 4. Third-party components

The Software incorporates open-source components listed in
[THIRD-PARTY-LICENSES.md](THIRD-PARTY-LICENSES.md). Those components remain
governed by their respective licenses.

## 5. Telemetry

Licensed use sends anonymous usage pings (command name, CLI version, and a
one-way fingerprint of the license key). No personal data, file content, or
machine identifiers are collected. See [docs/telemetry.md](docs/telemetry.md)
for the exact data and opt-out instructions.

## 6. Warranty disclaimer

THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR
IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY, FITNESS
FOR A PARTICULAR PURPOSE, AND NONINFRINGEMENT.

## 7. Limitation of liability

TO THE MAXIMUM EXTENT PERMITTED BY LAW, IN NO EVENT SHALL THE LICENSOR BE LIABLE
FOR ANY INDIRECT, INCIDENTAL, SPECIAL, OR CONSEQUENTIAL DAMAGES, OR FOR ANY LOSS
OF PROFITS OR DATA, ARISING OUT OF OR RELATED TO THE SOFTWARE.

## 8. Termination

This Agreement terminates automatically if You breach it. On termination You must
stop using and remove all copies of the Software. Sections 3, 6, 7, and 9 survive
termination.

## 9. Governing law

This Agreement is governed by the laws of the Licensor's jurisdiction, without
regard to conflict-of-law rules.

## 10. Contact

For licensing questions, seat counts, or team plans, see
[docs/licensing.md](docs/licensing.md) or the
[issues page](https://github.com/trellis-lab/trellis/issues).
