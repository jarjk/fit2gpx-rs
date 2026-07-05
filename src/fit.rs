use crate::{Res, utils};
use fit_file::{FitFieldValue, FitRecordMsg, fit_file};
use std::path::{Path, PathBuf};
use std::{fs, io};
use time::OffsetDateTime;

#[derive(Default, Clone)]
pub struct TrackSegment {
    pub points: Vec<TrackPoint>,
}

#[derive(Default, Clone, Debug)]
pub struct TrackPoint {
    pub lat: f64,
    pub lon: f64,
    pub elevation: Option<f64>,
    pub time: Option<OffsetDateTime>,
    pub speed: Option<f64>,
    pub heart_rate: Option<u8>,
    pub cadence: Option<u8>,
    pub temperature: Option<i8>,
    pub power: Option<u16>,
    pub distance: Option<f64>,
}

/// Fit Context structure. An instance of this will be passed to the parser and ultimately to the callback function so we can use it for whatever.
#[derive(Default, Clone)]
pub struct Fit {
    pub file_name: PathBuf,
    pub track_segment: TrackSegment,
}

// public fns
impl Fit {
    /// add a filename to `self`, create new instance
    #[must_use]
    pub fn with_filename(self, fname: impl Into<PathBuf>) -> Self {
        Fit {
            file_name: fname.into(),
            ..self
        }
    }
    /// create [`Fit`] from the fit file at `fit_path`
    /// # Errors
    /// can't open file at `fit_path`
    /// can't read fit: invalid
    pub fn from_file(fit_path: impl AsRef<Path>) -> Res<Self> {
        let mut file = fs::File::open(&fit_path)?;

        Ok(Self::from_reader(&mut file)?.with_filename(fit_path.as_ref()))
    }

    /// create [`Fit`] from the fit content of `reader`
    /// also deletes (probably) null or invalid `track_segment.points`
    /// # Errors
    /// can't read fit: invalid
    pub fn from_reader(reader: impl io::Read) -> Res<Self> {
        let mut fit = Fit::default();

        let mut bufread = io::BufReader::new(reader);
        fit_file::read(&mut bufread, Self::callback, &mut fit)?;

        fit.track_segment.points.retain(|wp| {
            !utils::is_00(wp) && (-90. ..90.).contains(&wp.lat) && (-180. ..180.).contains(&wp.lon)
        });
        Ok(fit)
    }
    /// convert a fit file at `fit_path`, write to `fname`
    /// # Errors
    /// can't read fit
    /// can't write gpx
    pub fn file_to_gpx(fit_path: impl AsRef<Path>, fname: impl AsRef<Path>) -> Res<()> {
        let fit = Fit::from_file(fit_path)?;
        fit.save_to_gpx(fname)
    }

    /// convert fit content from `read` to `fname`
    /// # Errors
    /// can't read fit
    /// can't write gpx
    pub fn reader_to_gpx(read: impl io::Read, fname: impl AsRef<Path>) -> Res<()> {
        let fit = Fit::from_reader(read)?;
        fit.save_to_gpx(fname)
    }

    /// write `self` to it's gpx, with the same filename, but gpx extension
    /// # Errors
    /// can't write gpx
    pub fn save_to_gpx(self, fname: impl AsRef<Path>) -> Res<()> {
        utils::write_gpx_to_file(&self.track_segment.points, fname)
    }
}
/// private fns
impl Fit {
    /// [`fit_file::FitRecordMsg`] to [`TrackPoint`]
    fn frm_to_gwp(frm: FitRecordMsg) -> TrackPoint {
        let time = frm.timestamp.unwrap_or(0);
        let time = OffsetDateTime::from_unix_timestamp(time.into()).ok();

        let lat = fit_file::semicircles_to_degrees(frm.position_lat.unwrap_or(0));
        let lon = fit_file::semicircles_to_degrees(frm.position_long.unwrap_or(0));

        let alt = if let Some(enh_alt) = frm.enhanced_altitude {
            Some(enh_alt)
        } else {
            frm.altitude.map(Into::into)
        }
        .map(|alt| alt as f32 / 5. - 500.); // m

        // let dist = frm.distance.unwrap_or(0) as f32 / 100000.; // km

        let speed = if let Some(enh_spd) = frm.enhanced_speed {
            Some(enh_spd)
        } else {
            frm.speed.map(Into::into)
        }
        .map(f64::from);
        let heart_rate = frm.heart_rate;
        let cadence = frm.cadence;
        let temperature = frm.temperature;
        let power = frm.power;
        let distance = frm.distance.map(f64::from);

        TrackPoint {
            lat,
            lon,
            elevation: alt.map(Into::into),
            time,
            speed,
            heart_rate,
            cadence,
            temperature,
            power,
            distance,
        }
    }
    /// Called for each record message as it is being processed.
    fn callback(
        timestamp: u32,
        global_message_num: u16,
        _local_msg_type: u8,
        _message_index: u16,
        fields: Vec<FitFieldValue>,
        data: &mut Fit,
    ) {
        // if global_message_num == fit_file::GLOBAL_MSG_NUM_SESSION {
        // let msg = FitSessionMsg::new(fields);
        // let sport_names = fit_file::init_sport_name_map();
        // let sport_id = msg.sport.unwrap();
        // println!("Sport: {}", sport_names.get(&sport_id).unwrap());
        // } else
        if global_message_num == fit_file::GLOBAL_MSG_NUM_RECORD {
            let mut msg = FitRecordMsg::new(fields);

            msg.timestamp = Some(timestamp);

            let wp = Self::frm_to_gwp(msg);
            data.track_segment.points.push(wp);
        }
    }
}
