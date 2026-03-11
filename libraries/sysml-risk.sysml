package SysMLRisk {
    doc /*
     * Domain library for risk management.
     *
     * Implements hazard analysis (MIL-STD-882E / ISO 14971) and
     * FMEA (AIAG/VDA, SAE J1739, IEC 60812) methodology.
     *
     * Scoring uses 1-5 integer scales:
     *   Severity:   1=Negligible  2=Marginal  3=Moderate  4=Critical  5=Catastrophic
     *   Occurrence: 1=Improbable  2=Remote    3=Occasional 4=Probable  5=Frequent
     *   Detection:  1=Almost Certain  2=High  3=Moderate  4=Low  5=Almost Impossible
     *
     * RPN = Severity x Occurrence x Detection (range 1-125).
     */

    private import ScalarValues::*;

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
         * A risk/hazard item tracked in the project risk register.
         *
         * Follows FMEA structure: each risk captures the failure mode,
         * its effect, its cause, current controls, and S/O/D scores.
         * Nest inside the part, action, or use case it applies to.
         */
        attribute failureMode : String;
        attribute failureEffect : String;
        attribute failureCause : String;
        attribute currentControls : String;
        attribute severity : Integer;
        attribute occurrence : Integer;
        attribute detection : Integer;
        attribute category : RiskCategory;
        attribute status : RiskStatus;
        attribute owner : String;
        attribute recommendedAction : String;
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
