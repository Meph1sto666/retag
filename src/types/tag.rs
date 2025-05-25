use super::errors;
use difflib::get_close_matches;
use leptess::tesseract::TessApi;
use opencv::{
    core::{Mat, Point, Rect, Size, Vector},
    imgcodecs,
    imgproc::{self, CHAIN_APPROX_SIMPLE},
    prelude::MatTraitConst,
};
use xcap::image::RgbaImage;

static COLOR_RGB: bool = true;
static RECRUITMENT_ROI_VERTICAL: (f64, f64) = (
    0.45, // ignore top 45%
    0.30, // ignore bottom 30%
);
static RECRUITMENT_ROI_HORIZONTAL: (f64, f64) = (
    0.3, // ignore left 30%
    0.3, // ignore right 30%
);
static MIN_TAG_BOX_SIZE: f64 = 0.005;
static MAX_TAG_BOX_SIZE: f64 = 0.250;
static SELECTED_ACCEPT_THRESH: f64 = 0.5;
static TAGS_STRINGS: [&str; 29] = [
    "Medic",
    "Caster",
    "Vanguard",
    "Guard",
    "Defender",
    "Defense",
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
pub enum TagType {
    Medic,
    Caster,
    Vanguard,
    Guard,
    Defender,
    Defense,
    Supporter,
    Melee,
    Debuff,
    FastRedeploy,
    Shift,
    Summon,
    Support,
    Survival,
    Elemental,
    Ranged,
    DpRecovery,
    Starter,
    Slow,
    AoE,
    Sniper,
    CrowdControl,
    Healing,
    DPS,
    Nuker,
    SeniorOperator,
    Specialist,
    Robot,
    TopOperator,
}

/// Converts a `TagType` to its corresponding string representation.
///
/// This implementation of the `ToString` trait allows for converting instances of the
/// `TagType` enum into their respective string representations. Each variant of the enum
/// is mapped to a specific string that describes the tag type.
///
/// # Returns
/// - `String`: The string representation of the `TagType` instance. Each variant is
///   converted to a corresponding string that describes the tag type.
///
/// # Example Usage
/// ```rust
/// let tag_type = TagType::Medic;
/// let tag_string = tag_type.to_string(); // "Medic"
/// println!("Tag type: {}", tag_string);
/// ```
///
/// # Notes
/// This implementation is useful for displaying tag types in user interfaces, logging,
/// or any situation where a string representation of the tag type is needed.
impl ToString for TagType {
    fn to_string(&self) -> String {
        match self {
            Self::Medic => "Medic".into(),
            Self::Caster => "Caster".into(),
            Self::Vanguard => "Vanguard".into(),
            Self::Guard => "Guard".into(),
            Self::Defender => "Defender".into(),
            Self::Defense => "Defense".into(),
            Self::Supporter => "Supporter".into(),
            Self::Melee => "Melee".into(),
            Self::Debuff => "Debuff".into(),
            Self::FastRedeploy => "Fast-Redeploy".into(),
            Self::Shift => "Shift".into(),
            Self::Summon => "Summon".into(),
            Self::Support => "Support".into(),
            Self::Survival => "Survival".into(),
            Self::Elemental => "Elemental".into(),
            Self::Ranged => "Ranged".into(),
            Self::DpRecovery => "Dp-Recovery".into(),
            Self::Starter => "Starter".into(),
            Self::Slow => "Slow".into(),
            Self::AoE => "AoE".into(),
            Self::Sniper => "Sniper".into(),
            Self::CrowdControl => "Crowd-Control".into(),
            Self::Healing => "Healing".into(),
            Self::DPS => "DPS".into(),
            Self::Nuker => "Nuker".into(),
            Self::SeniorOperator => "Senior-Operator".into(),
            Self::Specialist => "Specialist".into(),
            Self::Robot => "Robot".into(),
            Self::TopOperator => "Top-Operator".into(),
        }
    }
}

impl Clone for TagType {
	fn clone(&self) -> Self {
		match self {
			Self::Medic => Self::Medic,
			Self::Caster => Self::Caster,
			Self::Vanguard => Self::Vanguard,
			Self::Guard => Self::Guard,
			Self::Defender => Self::Defender,
			Self::Defense => Self::Defense,
			Self::Supporter => Self::Supporter,
			Self::Melee => Self::Melee,
			Self::Debuff => Self::Debuff,
			Self::FastRedeploy => Self::FastRedeploy,
			Self::Shift => Self::Shift,
			Self::Summon => Self::Summon,
			Self::Support => Self::Support,
			Self::Survival => Self::Survival,
			Self::Elemental => Self::Elemental,
			Self::Ranged => Self::Ranged,
			Self::DpRecovery => Self::DpRecovery,
			Self::Starter => Self::Starter,
			Self::Slow => Self::Slow,
			Self::AoE => Self::AoE,
			Self::Sniper => Self::Sniper,
			Self::CrowdControl => Self::CrowdControl,
			Self::Healing => Self::Healing,
			Self::DPS => Self::DPS,
			Self::Nuker => Self::Nuker,
			Self::SeniorOperator => Self::SeniorOperator,
			Self::Specialist => Self::Specialist,
			Self::Robot => Self::Robot,
			Self::TopOperator => Self::TopOperator,
		}
	}
}

/// Represents a tag detected in an image with associated properties.
///
/// The `Tag` struct encapsulates information about a tag, including its type, selection status,
/// and the bounding box that defines its location in the image. This struct is used in the context
/// of image processing and Optical Character Recognition (OCR) to manage and represent tags
/// extracted from images.
///
/// # Fields
/// - `tag_type`: The type of the tag, represented as a `TagType` enum. This field indicates
///   the specific category or role of the tag (e.g., Medic, Caster, Defender, etc.).
/// - `selected`: A boolean indicating whether the tag is selected. This can be used to track
///   user interactions or selections in a graphical user interface or processing logic.
/// - `bounding_box`: A `Rect` object representing the bounding box of the tag in the image.
///   This field defines the rectangular area that encompasses the tag, which is useful for
///   visualization and further processing.
///
/// # Example Usage
/// ```rust
/// let bounding_box = Rect::new(10, 10, 100, 50); // Define the bounding box
/// let tag = Tag {
///     tag_type: TagType::Medic, // Set the tag type
///     selected: true,           // Set the selection status
///     bounding_box,             // Use the defined bounding box
/// };
///
/// println!("Tag type: {:?}", tag.tag_type);
/// println!("Is selected: {}", tag.selected);
/// println!("Bounding box: {:?}", tag.bounding_box);
/// ```
///
/// # Notes
/// The `Tag` struct is typically created using the `Tag::new` method, which ensures that the
/// tag type is valid and handles any necessary initialization logic.
#[derive(Debug)]
pub struct Tag {
    tag_type: TagType,
    selected: bool,
    bounding_box: Rect,
}

/// Represents a tag detected in an image with associated properties.
///
/// The `Tag` struct encapsulates information about a tag, including its type, selection status,
/// and the bounding box that defines its location in the image. This struct is used in the context
/// of image processing and Optical Character Recognition (OCR) to manage and represent tags
/// extracted from images.
impl Tag {
    /// Creates a new `Tag` instance from the provided tag string, selection status, and bounding box.
    ///
    /// This constructor attempts to map the provided tag string to a corresponding `TagType`.
    /// If the tag string is invalid, it returns an error.
    ///
    /// # Parameters
    /// - `tag_string`: A string slice representing the tag's name. This string is used to
    ///   determine the type of the tag.
    /// - `selected`: A boolean indicating whether the tag is selected.
    /// - `bounding_box`: A reference to a `Rect` object that defines the bounding box of the
    ///   tag in the image.
    ///
    /// # Returns
    /// - `Result<Self, errors::TagError>`:
    ///   - On success, returns a new `Tag` instance.
    ///   - On failure, returns an error of type `errors::TagError` if the tag string is invalid.
    ///
    /// # Example Usage
    /// ```rust
    /// let tag_string = "Medic";
    /// let selected = true;
    /// let bounding_box = Rect::new(10, 10, 100, 50);
    /// match Tag::new(tag_string, selected, &bounding_box) {
    ///     Ok(tag) => {
    ///         println!("Created tag: {:?}", tag);
    ///     },
    ///     Err(e) => {
    ///         eprintln!("Error creating tag: {}", e);
    ///     }
    /// }
    /// ```
    fn new(
        tag_string: &str,
        selected: bool,
        bounding_box: &Rect,
    ) -> Result<Self, errors::TagError> {
        let tag_type = match tag_string {
            "Medic" => Ok(TagType::Medic),
            "Caster" => Ok(TagType::Caster),
            "Vanguard" => Ok(TagType::Vanguard),
            "Guard" => Ok(TagType::Guard),
            "Defender" => Ok(TagType::Defender),
            "Defense" => Ok(TagType::Defense),
            "Supporter" => Ok(TagType::Supporter),
            "Melee" => Ok(TagType::Melee),
            "Debuff" => Ok(TagType::Debuff),
            "Fast-Redeploy" | "FastRedeploy" | "Fast Redeploy" => Ok(TagType::FastRedeploy),
            "Shift" => Ok(TagType::Shift),
            "Summon" => Ok(TagType::Summon),
            "Support" => Ok(TagType::Support),
            "Survival" => Ok(TagType::Survival),
            "Elemental" => Ok(TagType::Elemental),
            "Ranged" => Ok(TagType::Ranged),
            "Dp-Recovery" | "DpRecovery" | "Dp Recovery" => Ok(TagType::DpRecovery),
            "Starter" => Ok(TagType::Starter),
            "Slow" => Ok(TagType::Slow),
            "AoE" => Ok(TagType::AoE),
            "Sniper" => Ok(TagType::Sniper),
            "Crowd-Control" | "CrowdControl" | "Crowd Control" => Ok(TagType::CrowdControl),
            "Healing" => Ok(TagType::Healing),
            "DPS" => Ok(TagType::DPS),
            "Nuker" => Ok(TagType::Nuker),
            "SeniorOperator" | "Senior-Operator" | "Senior Operator" => Ok(TagType::SeniorOperator),
            "Specialist" => Ok(TagType::Specialist),
            "Robot" => Ok(TagType::Robot),
            "Top-Operator" | "TopOperator" | "Top Operator" => Ok(TagType::TopOperator),
            _ => Err(errors::TagError::InvalidTagString),
        }?;
        Ok(Tag {
            tag_type: tag_type,
            selected: selected,
            bounding_box: bounding_box.clone(),
        })
    }

    /// Returns whether the tag is selected.
    ///
    /// # Returns
    /// - `bool`: `true` if the tag is selected, `false` otherwise.
    pub fn selected(&self) -> bool {
        self.selected
    }

    /// Returns the bounding box of the tag.
    ///
    /// This method retrieves the bounding box that defines the location of the tag in the image.
    /// The bounding box is represented as a `Rect` object, which includes the coordinates and
    /// dimensions of the rectangle encompassing the tag.
    ///
    /// # Returns
    /// - `Rect`: The bounding box of the tag, represented as a `Rect` object containing the
    ///   `x` and `y` coordinates of the top-left corner, along with the width and height.
    ///
    /// # Example Usage
    /// ```rust
    /// let bounding_box = tag.bounding_box(); // Get the bounding box of the tag
    /// println!("Bounding box: {:?}", bounding_box);
    /// ```
    ///
    /// # Notes
    /// The bounding box can be useful for visualizing the tag on the original image or for
    /// further processing tasks, such as collision detection or region analysis.
    pub fn bounding_box(&self) -> Rect {
        self.bounding_box
    }
    pub fn tag_type(&self) -> &TagType {
        &self.tag_type
    }
}

/// Determines if a specified region of an image is selected based on pixel intensity.
///
/// This function checks whether a given rectangular region in an image meets a certain
/// threshold for selection. It calculates the average pixel intensity of the region and
/// compares it against a predefined threshold to determine if the region is considered
/// "selected".
///
/// # Parameters
/// - `image`: A reference to a `Mat` object representing the input image. The image should
///   be in a format compatible with the OpenCV library (e.g., CV_8UC3 or CV_8UC1).
/// - `rect`: A reference to a `Rect` object that defines the region of interest in the
///   image. This rectangle is used to crop the image for analysis.
///
/// # Returns
/// - `Result<bool, Box<dyn std::error::Error>>`:
///   - On success, returns a boolean value indicating whether the region is selected (`true`)
///     or not (`false`).
///   - On failure, returns an error wrapped in a `Box` trait object, which can represent
///     any error type.
///
/// # Processing Steps
/// 1. **Region of Interest (ROI) Cropping**: The function crops the input image to the
///    specified rectangular region using the `roi` method.
///    
/// 2. **Pixel Intensity Calculation**: The function calculates the sum of pixel intensities
///    in the cropped region using the `sum_elems` function. This provides a measure of the
///    overall brightness of the region.
///    
/// 3. **Average Intensity Calculation**: The average pixel intensity is computed by dividing
///    the total intensity by the area of the cropped region. The area is obtained from the
///    size of the cropped image.
///    
/// 4. **Threshold Comparison**: The average intensity is compared against a predefined
///    threshold (`SELECTED_ACCEPT_THRESH`). If the average intensity meets or exceeds this
///    threshold, the region is considered selected.
///
/// # Example Usage
/// ```rust
/// let image: Mat = ...; // Load or create an image
/// let rect: Rect = ...; // Define the region of interest
/// match is_tag_region_selected(&image, &rect) {
///     Ok(selected) => {
///         if selected {
///             println!("The tag region is selected.");
///         } else {
///             println!("The tag region is not selected.");
///         }
///     },
///     Err(e) => {
///         eprintln!("Error checking tag region selection: {}", e);
///     }
/// }
/// ```
///
/// # Errors
/// This function may return errors related to image processing operations, such as issues
/// with cropping the image or calculating pixel intensities.
fn is_tag_region_selected(image: &Mat, rect: &Rect) -> Result<bool, Box<dyn std::error::Error>> {
    let cropped: opencv::boxed_ref::BoxedRef<'_, Mat> = image.roi(*rect)?;
    let total: f64 = opencv::core::sum_elems(&cropped)?
        .get(if COLOR_RGB {2} else {0})
        .unwrap()
        .to_owned();
    Ok((total / 255.0 / cropped.size().unwrap().area() as f64) >= SELECTED_ACCEPT_THRESH)
}

/// Extracts tags from an image using Optical Character Recognition (OCR).
///
/// This function processes an input image to detect regions that potentially contain tags,
/// extracts text from those regions using the Tesseract OCR engine, and creates a vector
/// of `Tag` objects based on the extracted text and selection status.
///
/// # Parameters
/// - `image`: A reference to a `Mat` object representing the input image from which tags
///   will be extracted. The image should be in a color format (e.g., CV_8UC3).
/// - `tesseract`: A mutable reference to a `TessApi` object, which is the Tesseract OCR
///   engine instance used for text recognition.
///
/// # Returns
/// - `Result<Vec<Tag>, Box<dyn std::error::Error>>`:
///   - On success, returns a vector of `Tag` objects representing the detected tags in the
///     image.
///   - On failure, returns an error wrapped in a `Box` trait object, which can represent
///     any error type.
///
/// # Processing Steps
/// 1. **Color Conversion**: The input image is converted from BGR color space to grayscale
///    using the `cvt_color` function. This simplifies the image and prepares it for tag
///    detection.
///    
/// 2. **Tag Box Detection**: The function calls `detect_tag_boxes` to identify potential
///    rectangular regions in the grayscale image that may contain tags. This function
///    returns a vector of rectangles representing the detected tag boxes.
///    
/// 3. **Tag Extraction**: For each detected rectangle:
///    - The function calls `tag_button_to_string` to extract text from the corresponding
///      region of the grayscale image. If no valid text is extracted, it continues to the
///      next rectangle.
///    - It checks if the tag region is selected using the `is_tag_region_selected` function.
///    - A new `Tag` object is created using the extracted text, selection status, and rectangle.
///      If the creation of the `Tag` fails, it is ignored.
///    
/// 4. **Return Tags**: After processing all detected rectangles, the function returns the
///    vector of `Tag` objects.
///
/// # Example Usage
/// ```rust
/// let image: Mat = ...; // Load or create an image
/// let mut tesseract: TessApi = ...; // Initialize Tesseract API
/// match image_to_tags(&image, &mut tesseract) {
///     Ok(tags) => {
///         for tag in tags {
///             println!("Detected tag: {:?}", tag);
///         }
///     },
///     Err(e) => {
///         eprintln!("Error extracting tags: {}", e);
///     }
/// }
/// ```
///
/// # Errors
/// This function may return errors related to image processing operations, memory allocation,
/// or OCR processing, such as issues with the input image format, problems with the Tesseract
/// API, or errors in tag creation.
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

/// Detects rectangular tag boxes in a given grayscale image.
///
/// This function processes a grayscale image to identify and return a vector of rectangles
/// that represent detected tag boxes. The detection is performed using image thresholding,
/// contour finding, and polygon approximation techniques. The function filters the detected
/// contours to ensure that only valid rectangular boxes within specified size constraints
/// are returned.
///
/// # Parameters
/// - `grayscale`: A reference to a `Mat` object representing the input grayscale image.
///   The image should be in a single-channel format (e.g., CV_8UC1).
///
/// # Returns
/// - `Result<Vec<Rect>, Box<dyn std::error::Error>>`:
///   - On success, returns a vector of `Rect` objects representing the bounding boxes of
///     detected tag boxes.
///   - On failure, returns an error wrapped in a `Box` trait object, which can represent
///     any error type.
///
/// # Processing Steps
/// 1. **Thresholding**: The input grayscale image is thresholded to create a binary image
///    where potential tag boxes are highlighted. A threshold value of 140 is used, and the
///    binary inversion is applied.
///    
/// 2. **Contour Detection**: The contours of the thresholded image are found using the
///    `findContours` function. The contours are stored in a vector for further processing.
///    
/// 3. **Polygon Approximation**: For each detected contour, the function approximates the
///    contour to a polygon. If the polygon has exactly four vertices, it is considered a
///    potential tag box.
///    
/// 4. **Bounding Box Filtering**: The bounding rectangle of the approximated polygon is
///    calculated. The function checks if the area of the bounding box is within specified
///    limits defined by `MIN_TAG_BOX_SIZE` and `MAX_TAG_BOX_SIZE`, relative to the area
///    of the input image. Only bounding boxes that meet these criteria are included in the
///    final result.
///
/// # Constants
/// - `MIN_TAG_BOX_SIZE`: A constant representing the minimum size ratio for valid tag boxes.
/// - `MAX_TAG_BOX_SIZE`: A constant representing the maximum size ratio for valid tag boxes.
///
/// # Example Usage
/// ```rust
/// let grayscale_image: Mat = ...; // Load or create a grayscale image
/// match detect_tag_boxes(&grayscale_image) {
///     Ok(tag_boxes) => {
///         for box in tag_boxes {
///             println!("Detected tag box: {:?}", box);
///         }
///     },
///     Err(e) => {
///         eprintln!("Error detecting tag boxes: {}", e);
///     }
/// }
/// ```
///
/// # Errors
/// This function may return errors related to image processing operations, such as
/// issues with the input image format or memory allocation failures.
fn detect_tag_boxes(grayscale: &Mat) -> Result<Vec<Rect>, Box<dyn std::error::Error>> {
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

/// Extracts text from a specified region of an image using Optical Character Recognition (OCR).
///
/// This function takes an image and a rectangle defining a region of interest (ROI),
/// crops the image to that region, applies thresholding, and then uses the Tesseract OCR
/// engine to extract text from the cropped image. The extracted text is then compared
/// against a predefined list of tag strings to find the closest match.
///
/// # Parameters
/// - `tess`: A mutable reference to a `TessApi` object, which is the Tesseract OCR engine
///   instance used for text recognition.
/// - `image`: A reference to a `Mat` object representing the input image from which text
///   will be extracted.
/// - `rect`: A reference to a `Rect` object that defines the region of interest in the
///   image. The rectangle is used to crop the image before performing OCR.
///
/// # Returns
/// - `Result<Option<String>, Box<dyn std::error::Error>>`:
///   - On success, returns an `Option<String>`. If a valid tag string is found, it returns
///     `Some(tag_string)`. If no valid tag string is found or if the extracted text is
///     too short, it returns `None`.
///   - On failure, returns an error wrapped in a `Box` trait object, which can represent
///     any error type.
///
/// # Processing Steps
/// 1. **Region of Interest (ROI) Calculation**: The function calculates a cropped region
///    based on the provided rectangle, adjusting the dimensions to create a margin around
///    the rectangle.
///    
/// 2. **Image Cropping**: The function crops the input image to the specified ROI using
///    the `roi` method.
///    
/// 3. **Thresholding**: A binary threshold is applied to the cropped image to enhance
///    the contrast between text and background, making it easier for the OCR engine to
///    recognize the text.
///    
/// 4. **Image Encoding**: The thresholded image is encoded into a TIFF format in memory,
///    which is then used as input for the Tesseract OCR engine.
///    
/// 5. **Text Extraction**: The function sets the encoded image to the Tesseract instance
///    and retrieves the recognized text as a UTF-8 string.
///    
/// 6. **Text Validation**: If the extracted text is shorter than three characters, the
///    function returns `None`.
///    
/// 7. **Tag Matching**: The extracted text is compared against a predefined list of tag
///    strings using the `get_close_matches` function. If a close match is found, it is
///    returned; otherwise, `None` is returned.
///
/// # Example Usage
/// ```rust
/// let mut tess: TessApi = ...; // Initialize Tesseract API
/// let image: Mat = ...; // Load or create an image
/// let rect: Rect = ...; // Define the region of interest
/// match tag_button_to_string(&mut tess, &image, &rect) {
///     Ok(Some(tag)) => {
///         println!("Detected tag: {}", tag);
///     },
///     Ok(None) => {
///         println!("No valid tag detected.");
///     },
///     Err(e) => {
///         eprintln!("Error during OCR: {}", e);
///     }
/// }
/// ```
///
/// # Errors
/// This function may return errors related to image processing operations, memory allocation,
/// or OCR processing, such as issues with the input image format or problems with the Tesseract
/// API.

fn tag_button_to_string(
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

pub fn into_mat(image: &RgbaImage) -> Mat {
    unsafe {
        Mat::new_rows_cols_with_data_unsafe_def(
            image.height() as i32,
            image.width() as i32,
            opencv::core::CV_8UC4,
            image.as_raw().clone().as_mut_ptr() as *mut _,
        )
        .unwrap()
    }
}

pub struct UiTag {
    tag_type: TagType,
    offset_x: i32,
    offset_y: i32,
    selected: bool,
    bounding_box: Rect,
}

impl UiTag {
    pub fn from_tag(tag: &Tag, off_x: i32, off_y: i32) -> Self {
        UiTag {
            tag_type: tag.tag_type().clone(),
            offset_x: off_x,
            offset_y: off_y,
            bounding_box: tag.bounding_box(),
            selected: tag.selected(),
        }
    }

    /// get the bounding box with the offset applied
    pub fn abs_bounding_box(&self) -> Rect {
        Rect::new(
            self.bounding_box.x + self.offset_x,
            self.bounding_box.y + self.offset_y,
            self.bounding_box.width,
            self.bounding_box.height,
        )
    }

	pub fn tag_type(&self) -> &TagType {
		&self.tag_type
	}
	pub fn selected(&self) -> bool {
		self.selected
	}
}
