use std::mem::MaybeUninit;

use bevy::prelude::*;
use bevy::utils::HashSet;
use bevy_xr::session::{session_running, status_changed_to, XrStatus};

use openxr::sys::{BaseOutStructure, SystemSpatialEntityPropertiesFB};
use openxr::Result;
use openxr::{raw::SceneFB, AsyncRequestIdFB};

use crate::resources::OxrFrameWaiter;
use crate::{
    init::OxrTrackingRoot,
    reference_space::{OxrPrimaryReferenceSpace, OxrReferenceSpace},
    resources::{OxrInstance, OxrScene, OxrSession, OxrSystemId, OxrTime},
};

pub struct OxrScenePlugin;

impl Plugin for OxrScenePlugin {
    fn build(&self, app: &mut App) {
        bevy::log::info!("Building ScenePlugin");
        let resources = app
            .world
            .get_resource::<OxrInstance>()
            .and_then(|instance| {
                app.world
                    .get_resource::<OxrSystemId>()
                    .map(|system_id| (instance, system_id))
            });

        if resources.is_some_and(|(instance, system_id)| {
            supports_scene(instance, *system_id).is_ok_and(|s| s)
        }) {
            // app.insert_resource::<OxrScene>(OxrScene);

            info!("SCENE | About to add systems!");

            app.add_systems(
                PreUpdate,
                // query_scene_anchors.run_if(status_changed_to(XrStatus::Running))
                query_scene_anchors.run_if(resource_added::<OxrPrimaryReferenceSpace>),
            );

            info!("Scene Understanding is supported with this runtime");
        } else {
            error!("Scene Understanding is not supported with this runtime");
        }
    }
}

/// Querying Anchors Captured for a Room
///
/// params:
/// - world: &mut [`World`](bevy::prelude::World)
pub fn query_scene_anchors(
    // instance: Res<OxrInstance>,
    // session: Res<OxrSession>,
    // default_ref_space: Res<OxrPrimaryReferenceSpace>,
    // reference_space: Query<Option<&OxrReferenceSpace>>,
    world: &mut World,
) {
    info!("SCENE | --------- Inside Query Scene Anchors ---------");
    // let ref_space = reference_space.single();
    //

    info!("SCENE | Query Scene Anchors");

    // let mut ref_space: Option<&OxrReferenceSpace> = None;
    // {
    //     let mut ref_space_query = world.query::<&OxrReferenceSpace>();
    //     if let Some(t) = ref_space_query.iter(&world).next() {
    //         info!("SCENE | Reference Space Found");
    //         ref_space = Some(t);
    //     } else {
    //         info!("SCENE | Reference Space Not Found");
    //     }
    // }

    let ref_space = world.get_resource::<OxrPrimaryReferenceSpace>().unwrap();
    let instance = world.get_resource::<OxrInstance>().unwrap();
    let session = world.get_resource::<OxrSession>().unwrap();

    // Step 1:  Query anchors with a room layout component.
    let storage_location_filter_info = openxr::sys::SpaceStorageLocationFilterInfoFB {
        ty: openxr::sys::SpaceStorageLocationFilterInfoFB::TYPE,
        next: std::ptr::null(),
        location: openxr::sys::SpaceStorageLocationFB::LOCAL,
    };

    let comp_filter_info = openxr::sys::SpaceComponentFilterInfoFB {
        ty: openxr::sys::SpaceComponentFilterInfoFB::TYPE,
        next: &storage_location_filter_info as *const _ as _,
        component_type: openxr::sys::SpaceComponentTypeFB::ROOM_LAYOUT,
    };

    let filter_base_header = openxr::sys::SpaceFilterInfoBaseHeaderFB {
        ty: openxr::sys::SpaceQueryInfoFB::TYPE,
        next: &comp_filter_info as *const _ as _,
    };

    let filter_header_ptr =
        &filter_base_header as *const _ as *const openxr::sys::SpaceFilterInfoBaseHeaderFB;

    // Value came from Meta's XrSamples, particularly `SceneModelXr.cpp`
    let max_result_count = 100;

    let space_query_info = openxr::sys::SpaceQueryInfoFB {
        ty: openxr::sys::SpaceQueryInfoFB::TYPE,
        next: std::ptr::null(),
        query_action: openxr::sys::SpaceQueryActionFB::LOAD,
        max_result_count,
        timeout: openxr::sys::Duration::from_nanos(0),
        exclude_filter: std::ptr::null(),
        filter: filter_header_ptr,
        // filter: std::ptr::null(),
    };

    let query_header_ptr =
        &space_query_info as *const _ as *const openxr::sys::SpaceQueryInfoBaseHeaderFB;

    let spatial_entity_query = instance
        .0
        .exts()
        .fb_spatial_entity_query
        .unwrap_or_else(|| {
            panic!("Spatial Entity Query not loaded");
        });

    let mut request_id = openxr::AsyncRequestIdFB::default();

    info!("SCENE | Before Query Spaces");
    info!("SCENE | Session: {:?}", request_id);

    info!("SCENE | query_header_ptr: {:#?}", query_header_ptr);

    unsafe {
        let _ = cvt((spatial_entity_query.query_spaces)(
            session.as_raw(),
            query_header_ptr,
            &mut request_id as *mut _,
        ));

        // let results = openxr::sys::SpaceQueryResultFB

        let mut space_query_results = openxr::sys::SpaceQueryResultsFB {
            ty: openxr::sys::SpaceQueryResultsFB::TYPE,
            next: std::ptr::null_mut(),
            result_capacity_input: 0, // 0 means we want to get the count
            result_count_output: 100,
            results: std::ptr::null_mut(),
        };

        // let mut space_query_results_ptr =
        //     openxr::sys::SpaceQueryResultsFB::out(&mut space_query_results as *mut _ as _);

        let mut space_query_results_ptr =
            openxr::sys::SpaceQueryResultsFB::out(&mut BaseOutStructure {
                ty: openxr::sys::SpaceQueryResultsFB::TYPE,
                next: std::ptr::null_mut(),
            });

        info!("SCENE | Before Retrieve Space Query Results");
        info!("SCENE | Request ID: {:?}", request_id);
        info!("SCENE | Space Query Results: {:#?}", space_query_results);
        info!(
            "SCENE | Space Query Results Ptr: {:#?}",
            space_query_results_ptr
        );

        let _results = cvt((spatial_entity_query.retrieve_space_query_results)(
            session.as_raw(),
            request_id,
            space_query_results_ptr.as_mut_ptr(),
            // &mut space_query_results as *mut _,
        ));

        info!("SCENE | After Retrieve Space Query Results");
        info!("SCENE | Space Query Results: {:#?}", space_query_results);
        info!(
            "SCENE | Space Query Results Ptr: {:#?}",
            space_query_results_ptr
        );
    }

    info!("SCENE | query_header_ptr: {:#?}", query_header_ptr);

    info!("SCENE | After Query Spaces");

    info!("SCENE | Request ID: {:?}", request_id);

    // Testing
    let mut test_uuid = openxr::sys::UuidEXT {
        data: [0; openxr::sys::UUID_SIZE_EXT],
    };

    let spatial_entity = instance.0.exts().fb_spatial_entity.unwrap();

    // let scene = instance.0.exts().fb_scene.unwrap();
    // let spatial_entity_query = instance.0.exts().fb_spatial_entity_query.unwrap();
    // let spatial_entity_storage = instance.0.exts().fb_spatial_entity_storage.unwrap();

    info!("SCENE | Starting Step 2");

    let mut wall_uuids = openxr::sys::UuidEXT {
        data: [0; openxr::sys::UUID_SIZE_EXT],
    };

    info!("SCENE | Default Wall UUIDs: {:?}", wall_uuids);

    if let Some(fb_scene) = instance.0.exts().fb_scene {
        info!("SCENE | fb_scene extension loaded");

        let mut room_layout = openxr::sys::RoomLayoutFB {
            ty: openxr::sys::RoomLayoutFB::TYPE,
            next: std::ptr::null(),
            floor_uuid: openxr::sys::UuidEXT { data: [0; 16] },
            ceiling_uuid: openxr::sys::UuidEXT { data: [0; 16] },
            wall_uuid_capacity_input: 0,
            wall_uuid_count_output: 25,
            // wall_uuids: &mut wall_uuids as *mut _,
            wall_uuids: std::ptr::null_mut(),
        };

        info!("SCENE | Before Get Space Room Layout: {:#?}", room_layout);

        // unsafe {
        //     let _ = cvt((fb_scene.get_space_room_layout)(
        //         session.as_raw(),
        //         ref_space.as_raw(),
        //         &mut room_layout as *mut _,
        //     ));
        // }

        // info!("SCENE | Room Layout Set!: {:#?}", room_layout);
        //
        // let mut wall_uuids = openxr::sys::UuidEXT {
        //     data: wall_uuids.data,
        // };
        //
        // room_layout.wall_uuids = &mut wall_uuids as *mut _;
        // room_layout.wall_uuid_capacity_input = wall_uuids.data.len() as _;

        unsafe {
            let _room_layout = cvt((fb_scene.get_space_room_layout)(
                session.as_raw(),
                ref_space.0.as_raw(),
                &mut room_layout as *mut _,
            ));

            info!("SCENE | Room Layout Set!: {:#?}", room_layout);

            room_layout.wall_uuids = room_layout.wall_uuids as *mut _;
            room_layout.wall_uuid_capacity_input = (*room_layout.wall_uuids).data.len() as _;

            info!("SCENE | Updated Room layout: {:#?}", room_layout);

            let _room_layout_2 = cvt((fb_scene.get_space_room_layout)(
                session.as_raw(),
                ref_space.as_raw(),
                &mut room_layout as *mut _,
            ));

            let mut uuid_set = HashSet::<String>::new();

            if room_layout.floor_uuid.data != [0; 16] {
                let uuid = room_layout
                    .floor_uuid
                    .data
                    .iter()
                    .fold(String::new(), |acc, x| acc + &format!("{:02x}", x));

                info!("SCENE | Floor UUID: {}", uuid);
                uuid_set.insert(uuid);
            } else {
                info!("SCENE | No Floor UUID Found");
            }
        }

        if room_layout.wall_uuid_count_output > 0 {
            info!("SCENE | Room Layout: {:#?}", room_layout);
        } else {
            info!("SCENE | No Room Layout Found");
        }
    } else {
        warn!("SCENE | fb_scene extension not loaded");
    }
}

pub fn locate_space() {}

#[inline]
pub fn supports_scene(instance: &OxrInstance, system: OxrSystemId) -> Result<bool> {
    if instance.exts().fb_scene.is_none() || instance.exts().fb_spatial_entity.is_none() {
        warn!("Scene Understanding Extension not loaded, Unable to create Scene Understanding!");
        return Ok(false);
    }

    unsafe {
        let mut scene = openxr::sys::SystemSpatialEntityPropertiesFB {
            ty: openxr::sys::SystemSpatialEntityPropertiesFB::TYPE,
            next: std::ptr::null(),
            supports_spatial_entity: openxr::sys::TRUE,
        };

        let mut props = openxr::sys::SystemProperties::out(&mut scene as *mut _ as _);
        cvt((instance.fp().get_system_properties)(
            instance.as_raw(),
            system.0,
            props.as_mut_ptr(),
        ))?;

        bevy::log::info!(
            "From supports_scene: Scene Understanding capabilities: {:?}",
            scene.supports_spatial_entity
        );

        info!("Scene: {:?}", scene);

        Ok(scene.supports_spatial_entity == openxr::sys::TRUE)
    }
}

fn cvt(x: openxr::sys::Result) -> openxr::Result<openxr::sys::Result> {
    if x.into_raw() >= 0 {
        Ok(x)
    } else {
        Err(x)
    }
}
