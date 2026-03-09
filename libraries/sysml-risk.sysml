package SysMLRisk {
    doc /*
     * Domain library for risk management.
     * Provides reusable definitions for identifying, assessing, and
     * mitigating risks across systems engineering programs.
     */

    private import ScalarValues::*;

    enum def SeverityLevel {
        doc /* Classifies the impact magnitude of a realized risk event. */
        enum negligible;
        enum marginal;
        enum moderate;
        enum critical;
        enum catastrophic;
    }

    enum def LikelihoodLevel {
        doc /* Qualitative probability that a risk event will occur. */
        enum improbable;
        enum remote;
        enum occasional;
        enum probable;
        enum frequent;
    }

    enum def DetectabilityLevel {
        doc /*
         * Ability to detect the risk or its precursors before impact.
         * Ordered from easiest to hardest to detect.
         */
        enum almostCertain;
        enum high;
        enum moderate;
        enum low;
        enum almostImpossible;
    }

    enum def RiskCategory {
        doc /* Domain classification for a risk item. */
        enum technical;
        enum schedule;
        enum cost;
        enum safety;
        enum regulatory;
        enum supplyChain;
        enum environmental;
    }

    enum def RiskStatus {
        doc /* Lifecycle state of a risk from identification through closure. */
        enum identified;
        enum analyzing;
        enum mitigating;
        enum monitoring;
        enum closed;
        enum accepted;
    }

    enum def MitigationStrategy {
        doc /* High-level approach for addressing a risk. */
        enum avoid;
        enum transfer;
        enum reduce;
        enum accept;
        enum contingency;
    }

    part def RiskDef {
        doc /*
         * A risk item tracked in the project risk register.
         * Combines identification metadata with qualitative
         * severity, likelihood, and detectability assessments.
         */
        attribute id : String;
        attribute title : String;
        attribute category : RiskCategory;
        attribute status : RiskStatus;
        attribute severity : SeverityLevel;
        attribute likelihood : LikelihoodLevel;
        attribute detectability : DetectabilityLevel;
        attribute owner : String;
        attribute identifiedDate : String;
    }

    action def MitigationDef {
        doc /*
         * An action taken to reduce or eliminate a risk.
         * Tracks ownership, schedule, and the targeted
         * effectiveness of the mitigation effort.
         */
        attribute description : String;
        attribute strategy : MitigationStrategy;
        attribute owner : String;
        attribute dueDate : String;
        attribute status : String;
        attribute effectivenessTarget : String;
    }

    connection def mitigates {
        doc /*
         * Links a mitigation action to the risk it addresses.
         * A single mitigation may target multiple risks and
         * a risk may have multiple mitigations.
         */
        end source : MitigationDef;
        end target : RiskDef;
    }

    connection def affects {
        doc /*
         * Links a risk to the system element it impacts.
         * The target end is intentionally untyped so that any
         * part, requirement, or other element can be referenced.
         */
        end source : RiskDef;
        end target;
    }
}
