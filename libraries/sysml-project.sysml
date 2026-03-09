package SysMLProject {
    doc /* Project management definitions for milestone gates and design reviews.
         * Use these types to define phase gates and readiness criteria
         * that the tool evaluates with 'sysml report gate'. */

    enum def PhaseDef {
        doc /* Standard product development phases. */
        enum concept;
        enum preliminaryDesign;
        enum detailedDesign;
        enum prototyping;
        enum designVerification;
        enum processValidation;
        enum production;
        enum sustaining;
    }

    enum def ReviewType {
        doc /* Standard design review types aligned with development phases. */
        enum SRR;
        enum PDR;
        enum CDR;
        enum TRR;
        enum FAR;
        enum PRR;
    }

    part def MilestoneDef {
        doc /* A project milestone with readiness criteria.
             * Define required coverage thresholds, maximum open risks,
             * and maximum open NCRs. The tool checks these with
             * 'sysml report gate'. */
        attribute phase : PhaseDef;
        attribute reviewType : ReviewType;
        attribute requiredCoverageThreshold : Real;
        attribute maxOpenCriticalRisks : Integer;
        attribute maxOpenNCRs : Integer;
    }
}
