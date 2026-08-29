enum CostNode {
    Resource {
        name: String,
        cost: u64,
    },
    Group {
        name: String,
        children: Vec<CostNode>,
    },
}
impl CostNode {
    fn resource(name: &str, monthly_cost_cents: u64) -> CostNode {
        Self::Resource {
            name: name.to_string(),
            cost: monthly_cost_cents,
        }
    }

    fn group(name: &str) -> CostNode {
        Self::Group {
            name: name.to_string(),
            children: Vec::new(),
        }
    }

    fn name(&self) -> &str {
        match self {
            Self::Resource { name, .. } | Self::Group { name, .. } => name,
        }
    }

    fn monthly_cost_cents(&self) -> u64 {
        match self {
            Self::Resource { cost, .. } => *cost,
            Self::Group { children, .. } => children.iter().map(CostNode::monthly_cost_cents).sum(),
        }
    }

    fn add_child(&mut self, child: Self) -> Result<(), String> {
        match self {
            Self::Group { children, .. } => {
                children.push(child);
                Ok(())
            }
            _ => Err("Cannot add child to non-group".to_string()),
        }
    }
}
fn main() {
    let mut production = CostNode::group("production");
    production
        .add_child(CostNode::resource("database", 12000))
        .unwrap();
    production
        .add_child(CostNode::resource("server", 8000))
        .unwrap();
    let mut staging = CostNode::group("staging");
    staging
        .add_child(CostNode::resource("server", 2000))
        .unwrap();
    let mut payments = CostNode::group("payments");
    payments.add_child(production).unwrap();
    payments.add_child(staging).unwrap();
    let total_cost = payments.monthly_cost_cents();
    println!("total cost cents: {}", total_cost);
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_resource_returns_cost() {
        let resource = CostNode::resource("test", 100);
        assert_eq!(resource.monthly_cost_cents(), 100);
    }

    #[test]
    fn test_empty_group_costs_nothing() {
        let group = CostNode::group("test");
        assert_eq!(group.monthly_cost_cents(), 0);
    }

    #[test]
    fn test_tree_total() {
        let mut production = CostNode::group("production");
        production
            .add_child(CostNode::resource("database", 12000))
            .unwrap();
        production
            .add_child(CostNode::resource("server", 8000))
            .unwrap();
        let mut staging = CostNode::group("staging");
        staging
            .add_child(CostNode::resource("server", 2000))
            .unwrap();
        let mut payments = CostNode::group("payments");
        payments.add_child(production).unwrap();
        payments.add_child(staging).unwrap();
        let total_cost = payments.monthly_cost_cents();
        assert_eq!(total_cost, 22_000);
    }

    #[test]
    fn test_add_child_to_resource_err() {
        let mut resource = CostNode::resource("test", 100);
        let result = resource.add_child(CostNode::resource("do not add", 100));
        assert!(result.is_err());
    }

    #[test]
    fn test_add_more_nested_groups() {
        let mut production = CostNode::group("production");
        production
            .add_child(CostNode::resource("database", 12000))
            .unwrap();
        production
            .add_child(CostNode::resource("server", 8000))
            .unwrap();
        let mut client_a = CostNode::group("client a");
        client_a
            .add_child(CostNode::resource("subscription", 1000))
            .unwrap();
        production.add_child(client_a);
        assert_eq!(production.monthly_cost_cents(), 21_000);
    }
}
