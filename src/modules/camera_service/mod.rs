//!
//! # Определение дефектов грузоподъемного каната по его изображениям
//! 
//! Алгоритм основан на анализе изображения каната, а точнее контуров каната
//! Для выявления дефектов изображение каната нормализуется (авто гамма, авто контраст),
//! затем изображение бинаризуется с использованием регулируемого значения порога бинаризации
//! 
//! ## Виды определяемых дфектов
//! - Расширение (симметричная деформация с увеличением среднего диаметра)
//! - Сужение (симметричная деформация с уменьшением среднего диаметра)
//! - Холмик (выпуклость на одной стороне изображения каната)
//! - Канавка (углубление на одной стороне изображения каната)
//! 
//! ## Процессс
//! - Из конфигурации получаем геометрию стрел, параметры настройки камер и сегментирования каната
//! ```yaml
//! boom:
//!     main-len: 5.3 m                                        # length of the main boom
//!     main-angle: point real 'App/Load.MainBoomAngle'        # degrees, current angle of the main boom to vertical axis
//!     rotary-len: 2.1 m                                      # length of the rotary boom
//!     rotary-angle: point real 'App/Load.RotaryBoomAngle'    # degrees, current angle of the rotary boom (jib) to boom axis
//! rope:
//!     width: 35 mm        # Diameter of the rome
//!     length: 3000 m      # Total working length of the rope
//!     segment: 100 mm     # Whole rope will divided by the segments for the Depreciation Rate calculation, use less to incrise accuracy
//!     pos: point real 'App/Winch.EncoderBR2'      # meters, current rope position
//!     load: point real '/App/Winch.Load'          # tonn, current rope load 
//! segment: 100 mm
//! segment-threshold: 5 mm
//! ```
//! - Канат с учетом заданных настроек условно нарезается на сенменты, размер сегмента должен быть 85..95% от ширины кадра (размер кадра вдоль каната)
//! - В процессе перемещения каната приложение получает ищменения длины вытравленной его части, 
//!     - Пересчитываем и получаем положение сегмента оносительно положения камеры
//! - В моменты когда камера проходит границу между соседними сегментами, берем кадр с камеры и считаем дефекты
//! - Граница между сегментами задается с допустимой погрешность `segment-threshold`
mod bf_match;
mod camera_service_conf;
mod camera_service;
mod image_conf;
mod template_match_conf;
mod template_match;

pub(crate) use bf_match::*;
pub(crate) use camera_service_conf::*;
pub(crate) use camera_service::*;
pub(crate) use image_conf::*;
pub(crate) use template_match_conf::*;
pub(crate) use template_match::*;
