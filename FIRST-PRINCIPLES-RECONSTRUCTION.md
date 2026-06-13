# First-Principles Reconstruction: DiskPeek

> Applied Elon Musk's first-principles thinking: break to fundamental truths, rebuild from zero.

## Core Problem

Users need to see what is eating their disk space with a visual breakdown.

## First Principles Breakdown

1. Disk scanning is a solved problem.
2. Visualization is the value. A treemap makes size data actionable.
3. The user wants to find big files and decide what to delete.

## Essential Features

| Priority | Feature |
|----------|---------|
| P0 | Scan directory tree and calculate sizes |
| P0 | Visual size breakdown (treemap) |
| P0 | Click to drill into subdirectories |
| P1 | Sort by size, type, date |

## Reconstruction Blueprint

300-line Rust binary. CLI scan -> JSON -> simple web view.

## Musk\'s Razor

Cut Tauri. Cut sunburst. Cut cache. Cut i18n. Rebuild as a focused 300-line tool.
