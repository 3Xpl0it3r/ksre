use std::ops::Deref;

use ratatui::Frame;

use ratatui::layout::Rect;

use crate::app::ui::util::debug_widget;
use crate::app::AppState;
use crate::kubernetes::api::PodDescribe;

pub fn draw_page_pod_status(
    f: &mut Frame,
    state: & AppState,
    pod_describe: Option<&PodDescribe>,
    area: Rect,
) {
    if pod_describe.is_none() {
        f.render_widget(debug_widget("Empty"), area);
        return;
    }
    let pod_describe = pod_describe.unwrap();
    let mut describe = Vec::new();
    /* describe.push(format!("Name:                    {}", pod_describe.name));
    describe.push(format!("Namespace:               {}", pod_describe.namespace));
    describe.push(format!("Priority:                {}", pod_describe.priority)); */
    unsafe {
        describe.push(format!(
            "Service Account:         {}",
            &(*pod_describe.service_account)
        ));
        describe.push(format!(
            "Node:                    {}",
            &(*pod_describe.node)
        ));
        describe.push(format!(
            "Start Time:              {}",
            pod_describe.start_time
        ));
        describe.push(format!(
            "Labels:                  {}",
            &(*pod_describe.labels)
        ));
        describe.push(format!(
            "Status:                  {}",
            &(*pod_describe.status)
        ));
        describe.push(format!("IP:                      {}", &(*pod_describe.ip)));
        describe.push("Containers".to_string());
        for container in pod_describe.containers.iter() {
            describe.push(format!(" {}", &(*container.name)));
            /* describe.push(format!("     ContainerId:        {}", container.container_id));
            describe.push(format!("     Image:              {}", container.image));
            describe.push(format!("     ImageId:            {}", container.image_id)); */
            for (k, v) in container.state.iter() {
                if (&(**k)).eq("State".to_string().as_str()) {
                    describe.push(format!("  {:<26}{:<16}", "State:", v));
                } else {
                    describe.push(format!("    {:<24}{:<16}", &(**k), v));
                }
            }
            for (k, v) in container.last_state.iter() {
                if (&(**k)).eq("State".to_string().as_str()) {
                    describe.push(format!("  {:<26}{:<16}", "Last State:", v));
                } else {
                    describe.push(format!("    {:<24}{:<16}", &(**k), v));
                }
            }
        }
    }
    /* describe.push("IPs:".to_string()); */
    /* for ip in pod_describe.ips.iter() {
        describe.push(format!(" IP:     {}", ip));
    } */

    /* describe.push("Conditions".to_string());
    describe.push(" Type            Status".to_string());
    for (k, v) in pod_describe.conditions.iter() {
        describe.push(format!(" {}              {}", k, v));
    }
    describe.push(format!("QOS Class:               {}", pod_describe.qos_class));
    describe.push(format!("Node-Selector:           {}", pod_describe.node_selector)); */

    f.render_widget(debug_widget(describe.join("\n").as_str()), area)
}
