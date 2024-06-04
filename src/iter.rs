use factorio_blueprint::{objects::Blueprint, Container};
use std::{marker::PhantomData, ptr};

pub trait ContainerExt {
    fn blueprints(&mut self) -> BlueprintsMut<'_>;
}

impl ContainerExt for Container {
    fn blueprints(&mut self) -> BlueprintsMut<'_> {
        BlueprintsMut {
            _phantom: PhantomData,
            recursive: None,
            index: 0,
            container: ptr::from_mut(self),
        }
    }
}

pub struct BlueprintsMut<'a> {
    _phantom: PhantomData<&'a ()>,
    container: *mut Container,
    index: usize,
    recursive: Option<Box<BlueprintsMut<'a>>>,
}

impl<'a> Iterator for BlueprintsMut<'a> {
    type Item = &'a mut Blueprint;

    fn next(&mut self) -> Option<Self::Item> {
        match unsafe { &mut *self.container } {
            Container::BlueprintBook(bpb) => {
                if let Some(recursive) = &mut self.recursive {
                    let rec = recursive.next();
                    if rec.is_some() {
                        return rec;
                    }
                    self.recursive = None;
                }
                let len = bpb.blueprints.len();
                while self.index < len {
                    let bp = &mut bpb.blueprints[self.index].item;
                    self.index += 1;
                    let mut new_iter = BlueprintsMut {
                        _phantom: PhantomData,
                        recursive: None,
                        index: 0,
                        container: ptr::from_mut(bp),
                    };
                    let item = new_iter.next();
                    if item.is_some() {
                        self.recursive = Some(Box::new(new_iter));
                        return item;
                    }
                }
                None
            }
            Container::Blueprint(bp) => {
                if self.index == 0 {
                    self.index += 1;
                    unsafe { Some(&mut *ptr::from_mut::<Blueprint>(bp)) }
                } else {
                    None
                }
            }
            _ => None,
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::iter::ContainerExt;
    use factorio_blueprint::{
        objects::{Blueprint, BlueprintBook, BlueprintBookBlueprintValue},
        Container,
    };

    #[test]
    fn single() {
        let mut bp = Container::Blueprint(Blueprint::default());
        let mut bpit = bp.blueprints();
        assert!(bpit.next().is_some(), "One blueprint");
        assert!(bpit.next().is_none(), "No more blueprints");
    }

    #[test]
    fn simple_book_one_item() {
        let mut bpb = Container::BlueprintBook(BlueprintBook {
            blueprints: vec![BlueprintBookBlueprintValue {
                index: 0,
                item: Container::Blueprint(Blueprint::default()),
            }],
            ..Default::default()
        });
        let mut bpit = bpb.blueprints();
        assert!(bpit.next().is_some(), "One blueprint");
        assert!(bpit.next().is_none(), "No more blueprints");
    }

    #[test]
    fn simple_book_more_items() {
        let mut bpb = Container::BlueprintBook(BlueprintBook {
            blueprints: vec![
                BlueprintBookBlueprintValue {
                    index: 0,
                    item: Container::Blueprint(Blueprint::default()),
                },
                BlueprintBookBlueprintValue {
                    index: 1,
                    item: Container::Blueprint(Blueprint::default()),
                },
                BlueprintBookBlueprintValue {
                    index: 2,
                    item: Container::Blueprint(Blueprint::default()),
                },
                BlueprintBookBlueprintValue {
                    index: 3,
                    item: Container::Blueprint(Blueprint::default()),
                },
            ],
            ..Default::default()
        });
        let mut bpit = bpb.blueprints();
        assert!(bpit.next().is_some(), "One blueprint");
        assert!(bpit.next().is_some(), "Two blueprint");
        assert!(bpit.next().is_some(), "Red blueprint");
        assert!(bpit.next().is_some(), "Blue blueprint");
        assert!(bpit.next().is_none(), "No more blueprints");
    }

    #[test]
    fn nested_book_one() {
        let mut bpb = Container::BlueprintBook(BlueprintBook {
            blueprints: vec![BlueprintBookBlueprintValue {
                index: 0,
                item: Container::BlueprintBook(BlueprintBook {
                    blueprints: vec![BlueprintBookBlueprintValue {
                        index: 0,
                        item: Container::Blueprint(Blueprint::default()),
                    }],
                    ..Default::default()
                }),
            }],
            ..Default::default()
        });
        let mut bpit = bpb.blueprints();
        assert!(bpit.next().is_some(), "One blueprint");
        assert!(bpit.next().is_none(), "No more blueprints");
    }

    #[test]
    fn nested_book_more() {
        let mut bpb = Container::BlueprintBook(BlueprintBook {
            blueprints: vec![BlueprintBookBlueprintValue {
                index: 0,
                item: Container::BlueprintBook(BlueprintBook {
                    blueprints: vec![
                        BlueprintBookBlueprintValue {
                            index: 0,
                            item: Container::Blueprint(Blueprint::default()),
                        },
                        BlueprintBookBlueprintValue {
                            index: 1,
                            item: Container::Blueprint(Blueprint::default()),
                        },
                    ],
                    ..Default::default()
                }),
            }],
            ..Default::default()
        });
        let mut bpit = bpb.blueprints();
        assert!(bpit.next().is_some(), "One blueprint");
        assert!(bpit.next().is_some(), "Two blueprint");
        assert!(bpit.next().is_none(), "No more blueprints");
    }

    #[test]
    fn multi_nested_book() {
        let mut bpb = Container::BlueprintBook(BlueprintBook {
            blueprints: vec![BlueprintBookBlueprintValue {
                index: 0,
                item: Container::BlueprintBook(BlueprintBook {
                    blueprints: vec![
                        BlueprintBookBlueprintValue {
                            index: 0,
                            item: Container::BlueprintBook(BlueprintBook {
                                blueprints: vec![BlueprintBookBlueprintValue {
                                    index: 0,
                                    item: Container::Blueprint(Blueprint::default()),
                                }],
                                ..Default::default()
                            }),
                        },
                        BlueprintBookBlueprintValue {
                            index: 1,
                            item: Container::Blueprint(Blueprint::default()),
                        },
                    ],
                    ..Default::default()
                }),
            }],
            ..Default::default()
        });
        let mut bpit = bpb.blueprints();
        assert!(bpit.next().is_some(), "One blueprint");
        assert!(bpit.next().is_some(), "Two blueprint");
        assert!(bpit.next().is_none(), "No more blueprints");
    }

    #[test]
    fn empty_book() {
        let mut bpb = Container::BlueprintBook(BlueprintBook {
            blueprints: vec![],
            ..Default::default()
        });
        let mut bpit = bpb.blueprints();
        assert!(bpit.next().is_none(), "No more blueprints");
    }

    #[test]
    fn multi_layer() {
        let mut bpb = Container::BlueprintBook(BlueprintBook {
            blueprints: vec![
                BlueprintBookBlueprintValue {
                    index: 0,
                    item: Container::Blueprint(Blueprint::default()),
                },
                BlueprintBookBlueprintValue {
                    index: 1,
                    item: Container::BlueprintBook(BlueprintBook {
                        blueprints: vec![
                            BlueprintBookBlueprintValue {
                                index: 0,
                                item: Container::Blueprint(Blueprint::default()),
                            },
                            BlueprintBookBlueprintValue {
                                index: 1,
                                item: Container::BlueprintBook(BlueprintBook {
                                    blueprints: vec![
                                        BlueprintBookBlueprintValue {
                                            index: 0,
                                            item: Container::BlueprintBook(BlueprintBook {
                                                blueprints: vec![],
                                                ..Default::default()
                                            }),
                                        },
                                        BlueprintBookBlueprintValue {
                                            index: 1,
                                            item: Container::Blueprint(Blueprint::default()),
                                        },
                                    ],
                                    ..Default::default()
                                }),
                            },
                            BlueprintBookBlueprintValue {
                                index: 0,
                                item: Container::Blueprint(Blueprint::default()),
                            },
                        ],
                        ..Default::default()
                    }),
                },
                BlueprintBookBlueprintValue {
                    index: 0,
                    item: Container::Blueprint(Blueprint::default()),
                },
            ],
            ..Default::default()
        });
        let mut bpit = bpb.blueprints();
        assert!(bpit.next().is_some(), "One blueprint");
        assert!(bpit.next().is_some(), "Two blueprint");
        assert!(bpit.next().is_some(), "Red blueprint");
        assert!(bpit.next().is_some(), "Blue blueprint");
        assert!(bpit.next().is_some(), "FIVE blueprints!?");
        assert!(bpit.next().is_none(), "No more blueprints");
    }
}
