// SPDX-License-Identifier: MPL-2.0
use super::*;

fn rect(x: u16, y: u16, width: u16, height: u16) -> Rect {
    Rect {
        x,
        y,
        width,
        height,
    }
}
fn positions(layout: &Layout, area: Rect) -> HashMap<usize, Rect> {
    let mut result = HashMap::new();
    layout.rectangles(area, &mut result);
    result
}

#[test]
fn expanding_a_middle_column_closes_a_fragmented_neighbour_and_fills_both_gaps() {
    let area = rect(7, 3, 90, 40);
    let mut layout = rebuild(
        area,
        &[
            (0, rect(7, 3, 90, 20)),
            (1, rect(7, 23, 30, 20)),
            (2, rect(37, 23, 30, 20)),
            (3, rect(67, 23, 30, 20)),
        ],
    )
    .unwrap();
    assert!(layout.expand_pane(2, area, rect(37, 3, 30, 40)));
    assert_eq!(
        positions(&layout, area),
        HashMap::from([
            (1, rect(7, 3, 30, 40)),
            (2, rect(37, 3, 30, 40)),
            (3, rect(67, 3, 30, 40)),
        ])
    );
}

#[test]
fn expansion_preserves_unaffected_boundaries_with_odd_extents_in_every_direction() {
    let area = rect(3, 7, 81, 41);
    let original = [
        (0, rect(3, 7, 81, 13)),
        (1, rect(3, 20, 27, 28)),
        (2, rect(30, 20, 54, 28)),
    ];
    // Reflect and transpose the same T junction, including non-zero origins.
    for transpose in [false, true] {
        for reflect in [false, true] {
            let transform = |mut r: Rect| {
                if reflect {
                    r.y = area.y + area.height - (r.y - area.y) - r.height;
                }
                if transpose {
                    r = rect(r.y, r.x, r.height, r.width);
                }
                r
            };
            let area = transform(area);
            let mut layout = rebuild(area, &original.map(|(id, r)| (id, transform(r)))).unwrap();
            let target = transform(rect(30, 7, 54, 41));
            assert!(layout.expand_pane(2, area, target));
            assert_eq!(
                positions(&layout, area),
                HashMap::from([
                    (0, transform(rect(3, 7, 27, 13))),
                    (1, transform(rect(3, 20, 27, 28))),
                    (2, target),
                ])
            );
        }
    }
}

#[test]
fn invalid_or_unchanged_expansion_leaves_the_split_tree_untouched() {
    let area = rect(0, 0, 80, 30);
    let mut layout = Layout::Pane(0);
    layout.split(0, 1, Axis::Horizontal);
    let before = format!("{layout:?}");
    for (pane, target) in [
        (0, rect(0, 0, 40, 30)),
        (0, rect(0, 0, 20, 30)),
        (0, rect(0, 0, 90, 30)),
        (9, area),
    ] {
        assert!(!layout.expand_pane(pane, area, target));
        assert_eq!(format!("{layout:?}"), before);
    }
}

#[test]
fn mixed_split_expansions_never_duplicate_panes_overlap_or_leave_gaps() {
    let screen = rect(5, 2, 101, 47);
    for pattern in 0usize..32 {
        let mut original = Layout::Pane(0);
        for id in 1..7 {
            let target = (pattern + id * 3) % id;
            let axis = if pattern & (1 << (id - 1)) == 0 {
                Axis::Horizontal
            } else {
                Axis::Vertical
            };
            original.split(target, id, axis);
        }
        let before = positions(&original, screen);
        for (&pane, &source) in &before {
            for target in [
                Rect {
                    x: screen.x,
                    width: right(source) - screen.x,
                    ..source
                },
                Rect {
                    width: right(screen) - source.x,
                    ..source
                },
                Rect {
                    y: screen.y,
                    height: bottom(source) - screen.y,
                    ..source
                },
                Rect {
                    height: bottom(screen) - source.y,
                    ..source
                },
            ] {
                let mut layout = original.clone();
                if !layout.expand_pane(pane, screen, target) {
                    assert_eq!(positions(&layout, screen), before);
                    continue;
                }
                let after = positions(&layout, screen);
                assert_eq!(after[&pane], target);
                let mut ids = Vec::new();
                layout.panes(&mut ids);
                assert_eq!(ids.len(), after.len(), "every pane has one rectangle");
                assert!(ids.iter().all(|id| before.contains_key(id)));
                assert_eq!(
                    after
                        .values()
                        .map(|r| u32::from(r.width) * u32::from(r.height))
                        .sum::<u32>(),
                    u32::from(screen.width) * u32::from(screen.height)
                );
                for (&id, &a) in &after {
                    assert_eq!(intersection(a, screen), Some(a));
                    for (&other, &b) in &after {
                        if id != other {
                            assert!(intersection(a, b).is_none());
                        }
                    }
                }
            }
        }
    }
}
