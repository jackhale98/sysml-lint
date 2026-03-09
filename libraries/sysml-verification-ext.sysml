package SysMLVerificationExt {
    doc /*
     * Domain library for verification extensions.
     * Provides reusable definitions for verification planning, execution,
     * and acceptance criteria in systems engineering workflows.
     */

    private import ScalarValues::*;

    enum def VerificationStatus {
        doc /* Tracks the lifecycle state of a verification activity. */
        enum notStarted;
        enum inProgress;
        enum pass;
        enum conditionalPass;
        enum fail;
        enum blocked;
        enum deferred;
    }

    enum def VerificationMethod {
        doc /* Identifies how a requirement is verified per standard V&V practice. */
        enum test;
        enum analysis;
        enum inspection;
        enum demonstration;
        enum simulation;
        enum review;
    }

    constraint def AcceptanceCriteriaDef {
        doc /*
         * Base definition for acceptance criteria that compare an actual
         * measured value against an expected value. Specialize this to
         * define project-specific pass/fail logic.
         */
        in actual : Real;
        in expected : Real;
    }

    constraint def WithinToleranceCriteria :> AcceptanceCriteriaDef {
        doc /*
         * Acceptance criterion that passes when the actual value falls
         * within a symmetric tolerance band around the expected value.
         * For example, with expected = 100 and tolerance = 5, any
         * actual value from 95 to 105 is acceptable.
         */
        in tolerance : Real;
        actual >= expected - tolerance and actual <= expected + tolerance
    }

    part def EquipmentDef {
        doc /*
         * Describes a piece of test or measurement equipment used
         * during verification. Captures calibration and accuracy data
         * needed for audit traceability.
         */
        attribute equipmentName : String;
        attribute calibrationDue : String;
        attribute accuracy : String;
    }

    action def TestProcedureStepDef {
        doc /*
         * A single step within a test procedure. Each step carries an
         * instruction for the operator, a flag indicating whether a
         * measurement is taken, and an optional safety warning.
         */
        attribute stepNumber : Integer;
        attribute instruction : String;
        attribute isMeasurement : Boolean;
        attribute safetyWarning : String;
    }

    part def VerificationExecutionDef {
        doc /*
         * Records the outcome of a single verification execution.
         * Captures who performed it, when, by what method, and
         * the resulting status for traceability reporting.
         */
        attribute executionDate : String;
        attribute operator : String;
        attribute status : VerificationStatus;
        attribute method : VerificationMethod;
        attribute notes : String;
    }
}
