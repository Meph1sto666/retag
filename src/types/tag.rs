use difflib::get_close_matches;
use leptess::tesseract::TessApi;
use opencv::{
    core::{Mat, Point, Rect, Size, Vector},
    imgcodecs,
    imgproc::{self, CHAIN_APPROX_SIMPLE},
    prelude::MatTraitConst,
};

use super::errors;

static MIN_TAG_BOX_SIZE: f64 = 0.005;
static MAX_TAG_BOX_SIZE: f64 = 0.250;
static SELECTED_ACCEPT_THRESH: f64 = 0.5;
static TAGS_STRINGS: [&str; 28] = [
    "Medic",
    "Caster",
    "Vanguard",
    "Guard",
    "Defender",
    "Supporter",
    "Melee",
    "Debuff",
    "Fast-Redeploy",
    "Shift",
    "Summon",
    "Support",
    "Survival",
    "Elemental",
    "Ranged",
    "Dp-Recovery",
    "Starter",
    "Slow",
    "AoE",
    "Sniper",
    "Crowd-Control",
    "Healing",
    "DPS",
    "Nuker",
    "Senior-Operator",
    "Specialist",
    "Robot",
    "Top-Operator",
];

#[derive(Debug)]
pub struct TagInfo {
    selected: bool,
    bounding_box: Rect,
}

#[derive(Debug)]
pub enum Tag {
    Medic(TagInfo),
    Caster(TagInfo),
    Vanguard(TagInfo),
    Guard(TagInfo),
    Defender(TagInfo),
    Supporter(TagInfo),
    Melee(TagInfo),
    Debuff(TagInfo),
    FastRedeploy(TagInfo),
    Shift(TagInfo),
    Summon(TagInfo),
    Support(TagInfo),
    Survival(TagInfo),
    Elemental(TagInfo),
    Ranged(TagInfo),
    DpRecovery(TagInfo),
    Starter(TagInfo),
    Slow(TagInfo),
    AoE(TagInfo),
    Sniper(TagInfo),
    CrowdControl(TagInfo),
    Healing(TagInfo),
    DPS(TagInfo),
    Nuker(TagInfo),
    SeniorOperator(TagInfo),
    Specialist(TagInfo),
    Robot(TagInfo),
    TopOperator(TagInfo),
}

trait TagInfoMethods {
    fn selected(&self) -> bool;
    fn bounding_box(&self) -> Rect;
}

impl TagInfoMethods for TagInfo {
    fn selected(&self) -> bool {
        self.selected
    }

    fn bounding_box(&self) -> Rect {
        self.bounding_box.clone() // Assuming Rect implements Clone
    }
}

impl ToString for Tag {
    fn to_string(&self) -> String {
        match self {
            Self::Medic(..) => "Medic".into(),
            Self::Caster(..) => "Caster".into(),
            Self::Vanguard(..) => "Vanguard".into(),
            Self::Guard(..) => "Guard".into(),
            Self::Defender(..) => "Defender".into(),
            Self::Supporter(..) => "Supporter".into(),
            Self::Melee(..) => "Melee".into(),
            Self::Debuff(..) => "Debuff".into(),
            Self::FastRedeploy(..) => "Fast-Redeploy".into(),
            Self::Shift(..) => "Shift".into(),
            Self::Summon(..) => "Summon".into(),
            Self::Support(..) => "Support".into(),
            Self::Survival(..) => "Survival".into(),
            Self::Elemental(..) => "Elemental".into(),
            Self::Ranged(..) => "Ranged".into(),
            Self::DpRecovery(..) => "Dp-Recovery".into(),
            Self::Starter(..) => "Starter".into(),
            Self::Slow(..) => "Slow".into(),
            Self::AoE(..) => "AoE".into(),
            Self::Sniper(..) => "Sniper".into(),
            Self::CrowdControl(..) => "Crowd-Control".into(),
            Self::Healing(..) => "Healing".into(),
            Self::DPS(..) => "DPS".into(),
            Self::Nuker(..) => "Nuker".into(),
            Self::SeniorOperator(..) => "Senior-Operator".into(),
            Self::Specialist(..) => "Specialist".into(),
            Self::Robot(..) => "Robot".into(),
            Self::TopOperator(..) => "Top-Operator".into(),
        }
    }
}
impl Tag {
    fn new(
        tag_string: &str,
        selected: bool,
        bounding_box: &Rect,
    ) -> Result<Self, errors::TagError> {
        let tag_info = TagInfo {
            selected: selected,
            bounding_box: bounding_box.clone(),
        };
        match tag_string {
            "Medic" => Ok(Self::Medic(tag_info)),
            "Caster" => Ok(Self::Caster(tag_info)),
            "Vanguard" => Ok(Self::Vanguard(tag_info)),
            "Guard" => Ok(Self::Guard(tag_info)),
            "Defender" => Ok(Self::Defender(tag_info)),
            "Supporter" => Ok(Self::Supporter(tag_info)),
            "Melee" => Ok(Self::Melee(tag_info)),
            "Debuff" => Ok(Self::Debuff(tag_info)),
            "Fast-Redeploy" | "FastRedeploy" | "Fast Redeploy" => Ok(Self::FastRedeploy(tag_info)),
            "Shift" => Ok(Self::Shift(tag_info)),
            "Summon" => Ok(Self::Summon(tag_info)),
            "Support" => Ok(Self::Support(tag_info)),
            "Survival" => Ok(Self::Survival(tag_info)),
            "Elemental" => Ok(Self::Elemental(tag_info)),
            "Ranged" => Ok(Self::Ranged(tag_info)),
            "Dp-Recovery" | "DpRecovery" | "Dp Recovery" => Ok(Self::DpRecovery(tag_info)),
            "Starter" => Ok(Self::Starter(tag_info)),
            "Slow" => Ok(Self::Slow(tag_info)),
            "AoE" => Ok(Self::AoE(tag_info)),
            "Sniper" => Ok(Self::Sniper(tag_info)),
            "Crowd-Control" | "CrowdControl" | "Crowd Control" => Ok(Self::CrowdControl(tag_info)),
            "Healing" => Ok(Self::Healing(tag_info)),
            "DPS" => Ok(Self::DPS(tag_info)),
            "Nuker" => Ok(Self::Nuker(tag_info)),
            "SeniorOperator" | "Senior-Operator" | "Senior Operator" => {
                Ok(Self::SeniorOperator(tag_info))
            }
            "Specialist" => Ok(Self::Specialist(tag_info)),
            "Robot" => Ok(Self::Robot(tag_info)),
            "Top-Operator" | "TopOperator" | "Top Operator" => Ok(Self::TopOperator(tag_info)),
            _ => Err(errors::TagError::InvalidTagString),
        }
    }

    fn selected(&self) -> bool {
        match self {
            Tag::Medic(i)
            | Tag::Caster(i)
            | Tag::Vanguard(i)
            | Tag::Guard(i)
            | Tag::Defender(i)
            | Tag::Supporter(i)
            | Tag::Melee(i)
            | Tag::Debuff(i)
            | Tag::FastRedeploy(i)
            | Tag::Shift(i)
            | Tag::Summon(i)
            | Tag::Support(i)
            | Tag::Survival(i)
            | Tag::Elemental(i)
            | Tag::Ranged(i)
            | Tag::DpRecovery(i)
            | Tag::Starter(i)
            | Tag::Slow(i)
            | Tag::AoE(i)
            | Tag::Sniper(i)
            | Tag::CrowdControl(i)
            | Tag::Healing(i)
            | Tag::DPS(i)
            | Tag::Nuker(i)
            | Tag::SeniorOperator(i)
            | Tag::Specialist(i)
            | Tag::Robot(i)
            | Tag::TopOperator(i) => i.selected,
        }
    }
    fn bounding_box(&self) -> Rect {
        match self {
            Tag::Medic(i)
            | Tag::Caster(i)
            | Tag::Vanguard(i)
            | Tag::Guard(i)
            | Tag::Defender(i)
            | Tag::Supporter(i)
            | Tag::Melee(i)
            | Tag::Debuff(i)
            | Tag::FastRedeploy(i)
            | Tag::Shift(i)
            | Tag::Summon(i)
            | Tag::Support(i)
            | Tag::Survival(i)
            | Tag::Elemental(i)
            | Tag::Ranged(i)
            | Tag::DpRecovery(i)
            | Tag::Starter(i)
            | Tag::Slow(i)
            | Tag::AoE(i)
            | Tag::Sniper(i)
            | Tag::CrowdControl(i)
            | Tag::Healing(i)
            | Tag::DPS(i)
            | Tag::Nuker(i)
            | Tag::SeniorOperator(i)
            | Tag::Specialist(i)
            | Tag::Robot(i)
            | Tag::TopOperator(i) => i.bounding_box,
        }
    }
}

fn is_tag_region_selected(image: &Mat, rect: &Rect) -> Result<bool, Box<dyn std::error::Error>> {
    let cropped: opencv::boxed_ref::BoxedRef<'_, Mat> = image.roi(*rect)?;
    let total: f64 = opencv::core::sum_elems(&cropped)?
        .get(0)
        .unwrap()
        .to_owned();
    Ok((total / 255.0 / cropped.size().unwrap().area() as f64) >= SELECTED_ACCEPT_THRESH)
}

pub fn image_to_tags(
    image: &Mat,
    mut tesseract: &mut TessApi,
) -> Result<Vec<Tag>, Box<dyn std::error::Error>> {
    let mut gray: Mat = Mat::default();
    _ = imgproc::cvt_color(
        &image,
        &mut gray,
        imgproc::COLOR_BGR2GRAY,
        0,
        opencv::core::AlgorithmHint::ALGO_HINT_ACCURATE,
    );

    let recs: Vec<Rect> = detect_tag_boxes(&gray)?;
    let mut tags: Vec<Tag> = vec![];
    for rec in recs {
        let tag_string: Option<String> = tag_button_to_string(&mut tesseract, &gray, &rec).unwrap();
        if tag_string.is_none() {
            continue;
        }
        let is_selected: bool = is_tag_region_selected(image, &rec)?;
        let tag: Result<Tag, errors::TagError> = Tag::new(&tag_string.unwrap(), is_selected, &rec);
        match tag {
            Ok(tag) => tags.push(tag),
            Err(_) => {}
        }
    }
    Ok(tags)
}

pub fn detect_tag_boxes(grayscale: &Mat) -> Result<Vec<Rect>, Box<dyn std::error::Error>> {
    let mut threshed: Mat = Mat::default();
    imgproc::threshold(
        &grayscale,
        &mut threshed,
        140.0,
        255.0,
        imgproc::THRESH_BINARY_INV,
    )?;
    let img_size: Size = threshed.size()?;
    let mut contours: Vector<Vector<Point>> = Vector::new();
    imgproc::find_contours_def(
        &threshed,
        &mut contours,
        imgproc::RETR_TREE,
        CHAIN_APPROX_SIMPLE,
    )?;
    let boxes: Vec<Rect> = contours
        .iter()
        .filter_map(|v: Vector<Point>| {
            let perimeter: f64 = imgproc::arc_length(&v, false).unwrap();
            let mut poly: Vector<Point> = Vector::new();
            _ = opencv::imgproc::approx_poly_dp(&v, &mut poly, 0.09 * perimeter, true);

            if poly.len() == 4 {
                let bounding: Rect = imgproc::bounding_rect(&poly).unwrap();

                if MAX_TAG_BOX_SIZE * (img_size.area() as f64) > bounding.area() as f64
                    && bounding.area() as f64 >= (MIN_TAG_BOX_SIZE * (img_size.area() as f64))
                {
                    Some(bounding)
                } else {
                    None
                }
            } else {
                None
            }
        })
        .collect();
    return Ok(boxes);
}

pub fn tag_button_to_string(
    tess: &mut TessApi,
    image: &Mat,
    rect: &Rect,
) -> Result<Option<String>, Box<dyn std::error::Error>> {
    let (x, y, w, h) = (
        rect.x + (0.05 * (rect.width as f64)) as i32,
        rect.y + (0.05 * (rect.height as f64)) as i32,
        rect.width - (0.1 * (rect.width as f64)) as i32,
        rect.height - (0.1 * (rect.height as f64)) as i32,
    );
    let cropped: opencv::boxed_ref::BoxedRef<'_, Mat> = image.roi(Rect::new(x, y, w, h))?;
    let mut threshed: Mat = Mat::default();
    imgproc::threshold(
        &cropped,
        &mut threshed,
        160.0,
        255.0,
        imgproc::THRESH_BINARY,
    )?;
    let mut buffer: Vector<u8> = Vector::new();
    imgcodecs::imencode(
        ".tiff",
        &threshed,
        &mut buffer,
        &opencv::core::Vector::new(),
    )?;
    let pix: leptess::leptonica::Pix = leptess::leptonica::pix_read_mem(&buffer.as_slice())?;
    tess.set_image(&pix);
    let tag_string: String = tess.get_utf8_text()?;

    if tag_string.len() < 3 {
        return Ok(None);
    }

    let a: Vec<&str> = get_close_matches(&tag_string, TAGS_STRINGS.into(), 1, 0.5);
    let v: Option<&&str> = a.get(0);
    if v.is_none() {
        return Ok(None);
    }
    let s: String = v.unwrap().to_string().clone();
    Ok(Some(s))
}
