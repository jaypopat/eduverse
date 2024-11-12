// Generated by dedot cli

import type { GenericSubstrateApi } from "dedot/types";
import type { AccountId32, Bytes } from "dedot/codecs";
import type {
  GenericContractEvents,
  GenericContractEvent,
} from "dedot/contracts";

export interface ContractEvents<ChainApi extends GenericSubstrateApi>
  extends GenericContractEvents<ChainApi> {
  /**
   *
   * @signature_topic: 0x7f569b8d80d7eb685bfebc5751d7f87fbf97d284f26472be03d69c60c5602575
   **/
  CourseCreated: GenericContractEvent<
    "CourseCreated",
    {
      /**
       *
       * @indexed: true
       **/
      courseId: number;
      /**
       *
       * @indexed: true
       **/
      teacher: AccountId32;
      /**
       *
       * @indexed: false
       **/
      title: Bytes;
    }
  >;

  /**
   *
   * @signature_topic: 0x4157fe48e6b2f8fb4a518ad7cd2fb7cc1b4c72fe285ed50083148a2e284cfe99
   **/
  StudentEnrolled: GenericContractEvent<
    "StudentEnrolled",
    {
      /**
       *
       * @indexed: true
       **/
      courseId: number;
      /**
       *
       * @indexed: true
       **/
      student: AccountId32;
    }
  >;
}