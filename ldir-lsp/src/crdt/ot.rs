/// Operational Transform for concurrent text editing.
/// Simpler fallback when full CRDT is not needed (e.g., 2 users).
/// Supports insert and delete operations with transform function.

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TextOperation {
    /// Insert text at position
    Insert {
        /// Byte position in the document
        pos: usize,
        /// Text to insert
        text: String,
    },
    /// Delete text at position
    Delete {
        /// Byte position in the document
        pos: usize,
        /// Number of bytes to delete
        len: usize,
    },
    /// No-op (identity)
    Retain {
        /// Number of bytes to retain
        len: usize,
    },
}

/// Transform two concurrent operations against each other.
/// Returns (op1', op2') such that applying op1' then op2' = applying op2 then op1'
pub fn transform(op1: &TextOperation, op2: &TextOperation) -> (TextOperation, TextOperation) {
    match (op1, op2) {
        (
            TextOperation::Insert { pos: p1, text: t1 },
            TextOperation::Insert { pos: p2, text: t2 },
        ) => {
            if *p1 <= *p2 {
                (
                    op1.clone(),
                    TextOperation::Insert {
                        pos: *p2 + t1.len(),
                        text: t2.clone(),
                    },
                )
            } else {
                (
                    TextOperation::Insert {
                        pos: *p1 + t2.len(),
                        text: t1.clone(),
                    },
                    op2.clone(),
                )
            }
        }

        (
            TextOperation::Insert { pos: p1, text: t1 },
            TextOperation::Delete { pos: p2, len: l2 },
        ) => {
            if *p1 <= *p2 {
                (
                    op1.clone(),
                    TextOperation::Delete {
                        pos: *p2 + t1.len(),
                        len: *l2,
                    },
                )
            } else if *p1 >= *p2 + *l2 {
                (
                    TextOperation::Insert {
                        pos: *p1 - *l2,
                        text: t1.clone(),
                    },
                    op2.clone(),
                )
            } else {
                let prefix_len = *p1 - *p2;
                let suffix_len = *l2 - prefix_len;
                (
                    op1.clone(),
                    TextOperation::Delete {
                        pos: *p2,
                        len: suffix_len,
                    },
                )
            }
        }

        (
            TextOperation::Delete { pos: p1, len: l1 },
            TextOperation::Insert { pos: p2, text: t2 },
        ) => {
            let (op2_t, op1_t) = transform(
                &TextOperation::Insert {
                    pos: *p2,
                    text: t2.clone(),
                },
                &TextOperation::Delete { pos: *p1, len: *l1 },
            );
            (op1_t, op2_t)
        }

        (
            TextOperation::Delete { pos: p1, len: l1 },
            TextOperation::Delete { pos: p2, len: l2 },
        ) => {
            let s1 = *p1;
            let e1 = *p1 + *l1;
            let s2 = *p2;
            let e2 = *p2 + *l2;

            if e1 <= s2 {
                (
                    op1.clone(),
                    TextOperation::Delete {
                        pos: *p2 - *l1,
                        len: *l2,
                    },
                )
            } else if e2 <= s1 {
                (
                    TextOperation::Delete {
                        pos: *p1 - *l2,
                        len: *l1,
                    },
                    op2.clone(),
                )
            } else {
                let new_end = e1.max(e2);
                let adjusted_start = if s1 <= s2 { s1 } else { s2 };
                let adjusted_len = new_end - adjusted_start;
                (
                    TextOperation::Delete {
                        pos: adjusted_start,
                        len: adjusted_len,
                    },
                    TextOperation::Retain { len: 0 },
                )
            }
        }

        (op1, TextOperation::Retain { .. }) => (op1.clone(), op2.clone()),
        (TextOperation::Retain { .. }, op2) => (op1.clone(), op2.clone()),
    }
}

/// Apply an operation to a string
pub fn apply_op(text: &str, op: &TextOperation) -> String {
    match op {
        TextOperation::Insert { pos, text: t } => {
            let mut result = text.to_string();
            result.insert_str(*pos, t);
            result
        }
        TextOperation::Delete { pos, len } => {
            let mut result = text.to_string();
            let end = (*pos + *len).min(result.len());
            result.drain(*pos..end);
            result
        }
        TextOperation::Retain { .. } => text.to_string(),
    }
}

/// Transform a list of operations against another list
pub fn transform_list(
    ops1: &[TextOperation],
    ops2: &[TextOperation],
) -> (Vec<TextOperation>, Vec<TextOperation>) {
    let mut t1 = ops1.to_vec();
    let mut t2 = ops2.to_vec();
    for op1 in ops1.iter() {
        for op2 in ops2.iter() {
            let (t1_new, t2_new) = transform(op1, op2);
            t1.push(t1_new);
            t2.push(t2_new);
        }
    }
    (t1, t2)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_insert_insert_before() {
        let op1 = TextOperation::Insert {
            pos: 0,
            text: "a".to_string(),
        };
        let op2 = TextOperation::Insert {
            pos: 2,
            text: "b".to_string(),
        };
        let (t1, t2) = transform(&op1, &op2);
        assert_eq!(
            t1,
            TextOperation::Insert {
                pos: 0,
                text: "a".to_string(),
            }
        );
        assert_eq!(
            t2,
            TextOperation::Insert {
                pos: 3,
                text: "b".to_string(),
            }
        );
    }

    #[test]
    fn test_insert_insert_after() {
        let op1 = TextOperation::Insert {
            pos: 5,
            text: "x".to_string(),
        };
        let op2 = TextOperation::Insert {
            pos: 2,
            text: "y".to_string(),
        };
        let (t1, t2) = transform(&op1, &op2);
        assert_eq!(
            t1,
            TextOperation::Insert {
                pos: 6,
                text: "x".to_string(),
            }
        );
        assert_eq!(
            t2,
            TextOperation::Insert {
                pos: 2,
                text: "y".to_string(),
            }
        );
    }

    #[test]
    fn test_insert_delete_before() {
        let op1 = TextOperation::Insert {
            pos: 1,
            text: "ab".to_string(),
        };
        let op2 = TextOperation::Delete { pos: 3, len: 2 };
        let (t1, t2) = transform(&op1, &op2);
        assert_eq!(
            t1,
            TextOperation::Insert {
                pos: 1,
                text: "ab".to_string(),
            }
        );
        assert_eq!(t2, TextOperation::Delete { pos: 5, len: 2 });
    }

    #[test]
    fn test_insert_delete_inside() {
        let op1 = TextOperation::Insert {
            pos: 5,
            text: "X".to_string(),
        };
        let op2 = TextOperation::Delete { pos: 3, len: 5 };
        let (t1, t2) = transform(&op1, &op2);
        assert_eq!(
            t1,
            TextOperation::Insert {
                pos: 5,
                text: "X".to_string(),
            }
        );
        assert_eq!(t2, TextOperation::Delete { pos: 3, len: 3 });
    }

    #[test]
    fn test_delete_delete_no_overlap() {
        let op1 = TextOperation::Delete { pos: 0, len: 2 };
        let op2 = TextOperation::Delete { pos: 5, len: 1 };
        let (t1, t2) = transform(&op1, &op2);
        assert_eq!(t1, TextOperation::Delete { pos: 0, len: 2 });
        assert_eq!(t2, TextOperation::Delete { pos: 3, len: 1 });
    }

    #[test]
    fn test_delete_delete_overlap() {
        let op1 = TextOperation::Delete { pos: 2, len: 4 };
        let op2 = TextOperation::Delete { pos: 4, len: 3 };
        let (t1, t2) = transform(&op1, &op2);
        assert_eq!(t1, TextOperation::Delete { pos: 2, len: 5 });
        assert_eq!(t2, TextOperation::Retain { len: 0 });
    }

    #[test]
    fn test_apply_op_insert() {
        let result = apply_op(
            "hello",
            &TextOperation::Insert {
                pos: 2,
                text: "XY".to_string(),
            },
        );
        assert_eq!(result, "heXYllo");
    }

    #[test]
    fn test_apply_op_delete() {
        let result = apply_op("hello", &TextOperation::Delete { pos: 1, len: 3 });
        assert_eq!(result, "ho");
    }

    #[test]
    fn test_transform_symmetry() {
        let op1 = TextOperation::Insert {
            pos: 0,
            text: "a".to_string(),
        };
        let op2 = TextOperation::Insert {
            pos: 0,
            text: "b".to_string(),
        };
        let (t1, t2) = transform(&op1, &op2);

        let base = "xyz";
        let via_original = apply_op(&apply_op(base, &op2), &op1);
        let via_transformed = apply_op(&apply_op(base, &t1), &t2);
        assert_eq!(
            via_original, via_transformed,
            "apply(op1') then op2') must equal apply(op2) then op1"
        );
    }
}
