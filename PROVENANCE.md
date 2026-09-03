# Provenance

This repository is a spec-first cleanroom implementation. Record here what was consulted.

## Specifications used
- Design Tokens Format Module 2025.10, W3C Design Tokens Community Group, Draft Community Group Report
  (https://www.designtokens.org/tr/drafts/format/; the first stable version, published 2025-10-28): the token
  object (`$value`, `$type`, `$description`, `$extensions`, `$deprecated`), the group and its `$type`
  inheritance, the name rules (no leading `$`, no `.`, `{` or `}`), the rule that a token with no resolvable
  type is invalid and no type is inferred, the `{group.token}` alias syntax and the ban on circular references,
  the `dimension` type (`value`, `unit` in `px` or `rem`), the media type and file extensions. The specification
  `pub-design-system-tokens` implements.
- Design Tokens Color Module 2025.10, W3C Design Tokens Community Group, Draft Community Group Report
  (https://www.designtokens.org/tr/drafts/color/): the `color` value object (`colorSpace`, `components`,
  `alpha`, `hex`), the fourteen colour spaces and their component ranges, the `none` keyword, the default alpha
  of 1, the six-digit hex fallback. The colour model of `pub-design-system-tokens`.

## Behavioural references (cited, not copied)
- _none_ — no token tool's source was opened; the two documents above are the whole reference.

## Copyleft sources
None consulted. Contributors who have studied GPL/AGPL implementations of this domain do not author the corresponding modules (two-team rule; see the Charter §09).

## AI assistance
Prompts point at the specifications and conformance suites above, never at copyleft source. Generated code is reviewed against this list before merge.
