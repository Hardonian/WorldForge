//! # worldforge-agent
//!
//! Deterministic agent system for World Forge.
//! Provides rule-based and utility-based agents that make decisions
//! using only deterministic RNG streams.
//!
//! The `CognitiveProvider` trait defines the future extension point
//! for LLM-backed agents, but only deterministic providers are implemented.

use serde::{Deserialize, Serialize};
use worldforge_core::rng::DeterministicRng;
use worldforge_core::Fixed64;

/// An action an agent can take.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentAction {
    pub action_type: String,
    pub target: Option<String>,
    pub params: std::collections::BTreeMap<String, String>,
}

/// Context provided to an agent for decision-making.
#[derive(Debug, Clone)]
pub struct AgentContext {
    pub tick: u64,
    pub observations: std::collections::BTreeMap<String, Fixed64>,
}

/// Trait for agent decision-making policies.
pub trait AgentPolicy: Send + Sync {
    fn name(&self) -> &str;
    fn decide(&mut self, context: &AgentContext, rng: &mut DeterministicRng) -> Vec<AgentAction>;
}

/// A simple rule-based agent using if/then conditions.
pub struct RuleBasedAgent {
    name: String,
    rules: Vec<Rule>,
}

/// A single rule: if condition is met, produce an action.
#[derive(Debug, Clone)]
pub struct Rule {
    pub condition: RuleCondition,
    pub action: AgentAction,
}

/// Conditions that can be checked.
#[derive(Debug, Clone)]
pub enum RuleCondition {
    /// Check if a named observation is below a threshold.
    Below { key: String, threshold: Fixed64 },
    /// Check if a named observation is above a threshold.
    Above { key: String, threshold: Fixed64 },
    /// Always true.
    Always,
}

impl RuleCondition {
    pub fn evaluate(&self, observations: &std::collections::BTreeMap<String, Fixed64>) -> bool {
        match self {
            RuleCondition::Below { key, threshold } => {
                observations.get(key).copied().unwrap_or(Fixed64::ZERO) < *threshold
            }
            RuleCondition::Above { key, threshold } => {
                observations.get(key).copied().unwrap_or(Fixed64::ZERO) > *threshold
            }
            RuleCondition::Always => true,
        }
    }
}

impl RuleBasedAgent {
    pub fn new(name: &str) -> Self {
        Self {
            name: name.to_string(),
            rules: Vec::new(),
        }
    }

    pub fn add_rule(&mut self, rule: Rule) {
        self.rules.push(rule);
    }
}

impl AgentPolicy for RuleBasedAgent {
    fn name(&self) -> &str {
        &self.name
    }

    fn decide(&mut self, context: &AgentContext, _rng: &mut DeterministicRng) -> Vec<AgentAction> {
        let mut actions = Vec::new();
        for rule in &self.rules {
            if rule.condition.evaluate(&context.observations) {
                actions.push(rule.action.clone());
            }
        }
        actions
    }
}

/// A utility-based agent that scores possible actions and picks the best.
pub struct UtilityAgent {
    name: String,
    options: Vec<UtilityOption>,
}

/// A possible action with a scoring function.
#[derive(Clone)]
pub struct UtilityOption {
    pub action: AgentAction,
    pub score_fn: fn(&AgentContext) -> Fixed64,
}

impl UtilityAgent {
    pub fn new(name: &str) -> Self {
        Self {
            name: name.to_string(),
            options: Vec::new(),
        }
    }

    pub fn add_option(&mut self, option: UtilityOption) {
        self.options.push(option);
    }
}

impl AgentPolicy for UtilityAgent {
    fn name(&self) -> &str {
        &self.name
    }

    fn decide(&mut self, context: &AgentContext, _rng: &mut DeterministicRng) -> Vec<AgentAction> {
        if self.options.is_empty() {
            return vec![];
        }

        let mut best_score = Fixed64::MIN;
        let mut best_action = None;

        for option in &self.options {
            let score = (option.score_fn)(context);
            if score > best_score {
                best_score = score;
                best_action = Some(option.action.clone());
            }
        }

        best_action.into_iter().collect()
    }
}

/// Future extension point for cognitive (LLM) providers.
/// Only deterministic providers are implemented now.
pub trait CognitiveProvider: Send + Sync {
    fn provider_type(&self) -> CognitiveProviderType;
    fn decide(
        &mut self,
        context: &AgentContext,
        rng: &mut DeterministicRng,
    ) -> Result<Vec<AgentAction>, String>;
}

/// Types of cognitive providers.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum CognitiveProviderType {
    Deterministic,
    /// Future: local LLM (e.g., Ollama)
    LocalLlm,
    /// Future: remote LLM API
    RemoteLlm,
}

/// The default deterministic cognitive provider.
pub struct DeterministicProvider {
    policy: Box<dyn AgentPolicy>,
}

impl DeterministicProvider {
    pub fn new(policy: Box<dyn AgentPolicy>) -> Self {
        Self { policy }
    }
}

impl CognitiveProvider for DeterministicProvider {
    fn provider_type(&self) -> CognitiveProviderType {
        CognitiveProviderType::Deterministic
    }

    fn decide(
        &mut self,
        context: &AgentContext,
        rng: &mut DeterministicRng,
    ) -> Result<Vec<AgentAction>, String> {
        Ok(self.policy.decide(context, rng))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rule_based_agent() {
        let mut agent = RuleBasedAgent::new("test_agent");
        agent.add_rule(Rule {
            condition: RuleCondition::Below {
                key: "stock".to_string(),
                threshold: Fixed64::from_int(10),
            },
            action: AgentAction {
                action_type: "reorder".to_string(),
                target: Some("warehouse".to_string()),
                params: std::collections::BTreeMap::new(),
            },
        });

        let mut rng = DeterministicRng::new(42, "test");
        let mut observations = std::collections::BTreeMap::new();
        observations.insert("stock".to_string(), Fixed64::from_int(5));

        let context = AgentContext {
            tick: 0,
            observations,
        };

        let actions = agent.decide(&context, &mut rng);
        assert_eq!(actions.len(), 1);
        assert_eq!(actions[0].action_type, "reorder");
    }

    #[test]
    fn agent_decisions_are_deterministic() {
        let build_agent = || {
            let mut agent = RuleBasedAgent::new("test");
            agent.add_rule(Rule {
                condition: RuleCondition::Always,
                action: AgentAction {
                    action_type: "idle".to_string(),
                    target: None,
                    params: std::collections::BTreeMap::new(),
                },
            });
            agent
        };

        let mut agent1 = build_agent();
        let mut agent2 = build_agent();
        let mut rng1 = DeterministicRng::new(42, "agent");
        let mut rng2 = DeterministicRng::new(42, "agent");

        let context = AgentContext {
            tick: 0,
            observations: std::collections::BTreeMap::new(),
        };

        let a1 = agent1.decide(&context, &mut rng1);
        let a2 = agent2.decide(&context, &mut rng2);
        assert_eq!(a1.len(), a2.len());
    }
}
