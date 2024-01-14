use std::vec;

use crate::widget::{Canvas, CutDir, Tab, Widget};

#[derive(Default, Clone, Copy, PartialEq, Eq)]
enum MainTabs {
    #[default]
    Explore,
    Tuning,
}
impl Tab for MainTabs {
    type Iterator = vec::IntoIter<Self>;
    fn iter() -> Self::Iterator {
        vec![MainTabs::Explore, MainTabs::Tuning].into_iter()
    }
    fn name(&self) -> &str {
        match self {
            MainTabs::Explore => "Explore",
            MainTabs::Tuning => "Tuning",
        }
    }
}

#[derive(Default)]
pub struct Main {
    tab: MainTabs,
}

impl Widget for Main {
    fn draw(&mut self, canvas: &mut Canvas) {
        canvas.visuals.dir = CutDir::Vertical;

        canvas.cut_top(canvas.font_height(), |canvas| {
            canvas.visuals.dir = CutDir::Horizontal;
            canvas.tabs(&mut self.tab);
        });

        canvas.center(
            38 * canvas.visuals.font.width * 2,
            canvas.visuals.font.height * 2 * 3,
            |canvas| {
                canvas.text("C  C♯  C♭  C♮  C𝄪  C𝄫  C𝄲  C𝄳  C𝄲♯  C𝄳♭ ");
                canvas.text("C7 C♯7 C♭7 C♮7 C𝄪7 C𝄫7 C𝄲7 C𝄳7 C𝄲♯7 C𝄳♭7");
                canvas.text("Cm C♯m C♭m C♮m C𝄪m C𝄫m C𝄲m C𝄳m C𝄲♯m C𝄳♭m");
            },
        );
    }
}
