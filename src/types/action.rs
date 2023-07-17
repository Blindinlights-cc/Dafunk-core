use serde::{de::DeserializeOwned, Deserialize, Serialize};
use serde_json::Value;
pub trait ActionParams: Serialize {
    const NAME: &'static str;
    type Response: DeserializeOwned + Clone;
}
use super::Selft;
///  
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct OnebotAction<T> {
    pub action: String,
    pub params: T,
    pub echo: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(rename = "self")]
    pub self_: Option<Selft>,
}
pub trait Action {
    type ActionResponse;
    fn name(&self) -> &'static str;
    fn json(&self) -> String;
    fn bytes(&self) -> Vec<u8>;
    fn self_type(&self) -> Option<&Selft>;
    fn echo(&self) -> String;
}
impl<T: ActionParams> Action for OnebotAction<T> {
    type ActionResponse = T::Response;
    fn name(&self) -> &'static str {
        T::NAME
    }
    fn json(&self) -> String {
        serde_json::to_string(&self).unwrap()
    }
    fn bytes(&self) -> Vec<u8> {
        serde_json::to_vec(&self).unwrap()
    }
    fn self_type(&self) -> Option<&Selft> {
        self.self_.as_ref()
    }
    fn echo(&self) -> String {
        self.echo.clone()
    }
}

impl<T: ActionParams> OnebotAction<T> {
    pub fn new(params: T, echo: String) -> Self {
        Self {
            action: T::NAME.to_string(),
            params,
            echo,
            self_: None,
        }
    }
    pub fn self_type(mut self, selft: Selft) -> Self {
        self.self_ = Some(selft);
        self
    }
}

macro_rules! impl_payload {
    (
        @[name=$name:ident]
        $(
            #[ $($method_meta:tt)* ]
        )*
        $vi:vis $Action:ident  => $Ret:ty {
            $(
                required {
                    $(
                        $(
                            #[ $($field_meta:tt)* ]
                        )*
                        $v:vis $fields:ident : $FTy:ty
                        ,
                    )*
                }
            )?

            $(
                optional {
                    $(
                        $(
                            #[ $($opt_field_meta:tt)* ]
                        )*
                        $opt_v:vis $opt_fields:ident : $OptFTy:ty
                    ),*
                    $(,)?
                }
            )?
        }
        $(
            $(
                #[ $($response_meta:tt)* ]
            )*
            $vr:vis $Response:ident
            {

                $(
                    $(
                        #[ $($resp_field_meta:tt)* ]
                    )*
                    $resp_v:vis $resp_fields:ident : $RespFTy:ty
                    ,
                )*

            }
        )?

    ) => {
        $(
            #[ $($method_meta)* ]
        )*
        $vi struct $Action {
            $(
                $(
                    $(
                        #[ $($field_meta)* ]
                    )*
                    $v $fields : $FTy,
                )*
            )?
            $(
                $(
                    $(
                        #[ $($opt_field_meta)* ]
                    )*
                    $opt_v $opt_fields : core::option::Option<$OptFTy>,
                )*
            )?
        }
        $(
            $(
                #[ $($response_meta)* ]
            )*
            $vr struct $Response {
                $(
                    $(
                        #[ $($resp_field_meta)* ]
                    )*
                    $resp_v $resp_fields : $RespFTy,
                )*

            }
        )?



        impl $crate::types::action::ActionParams for $Action {
            const NAME: &'static str = stringify!($name);
            type Response = $Ret;
        }

    };
}

impl_payload!(
    @[name=get_latest_events]
    #[derive(Debug, Clone, PartialEq, Serialize)]
    pub GetLatestEvents => Vec<Value> {
        required {
            pub limit: i64,
            pub timeout: i64,
        }
    }
);
impl_payload!(
    @[name=get_status]
    #[derive(Debug, Clone, PartialEq, Serialize)]
    pub GetStatus =>BotsStatus {


    }

    #[derive(Debug, Clone, PartialEq, Deserialize)]
    pub BotsStatus{
        pub good: bool,
        pub bots: Vec<crate::types::BotsInfo>,
    }
);

impl_payload!(
    @[name=get_supported_actions]
    #[derive(Debug, Clone, PartialEq, Serialize)]
    pub GetSupportedActions => Vec<String> {}
);
impl_payload!(
    @[name=get_version_info]
    #[derive(Debug, Clone, PartialEq, Serialize)]
    pub GetVersionInfo => VersionInfo {}

    #[derive(Debug, Clone, PartialEq, Deserialize)]
    pub VersionInfo {
        #[serde(rename = "impl")]
        pub impl_: String,
        pub version: String,
        pub author: String,
        pub repo: String,
    }
);

impl_payload!(
    @[name=send_message]
    #[derive(Debug, Clone, PartialEq, Serialize)]
    pub SendPrivateMsg => SendMessageResponse {
        required {
            pub detailed_type: String,
            pub message: crate::types::segment::MessageSegments,
        }
        optional {
            pub user_id: String,
            pub group_id: String,
            pub guild_id: String,
            pub channel_id: String,
        }
    }

    #[derive(Debug, Clone, PartialEq, Deserialize)]
    pub SendMessageResponse {
        pub message_id: String,
        pub time: i64,
    }
);

impl_payload!(
    @[name=delete_message]
    #[derive(Debug, Clone, PartialEq, Serialize)]
    pub DeleteMessage => () {
        required {
            pub message_id: String,
        }
    }
);
impl_payload!(
    @[name=get_self_info]
    #[derive(Debug, Clone, PartialEq, Serialize)]
    pub GetSelfInfo => SelfInfo {}

    #[derive(Debug, Clone, PartialEq, Deserialize)]
    pub SelfInfo{
        pub user_id: String,
        pub nickname: String,
        pub user_display_name: String,
    }
);
impl_payload!(
    @[name=get_user_info]
    #[derive(Debug, Clone, PartialEq, Serialize)]
    pub GetUserInfo => UserInfo {
        required {
            pub user_id: String,
        }
    }

    #[derive(Debug, Clone, PartialEq, Deserialize)]

    pub UserInfo{
        pub user_id: String,
        pub nickname: String,
        pub user_display_name: String,
        pub user_remark:String,
    }
);
impl_payload!(
    @[name=get_friend_list]
    #[derive(Debug, Clone, PartialEq, Serialize)]
    pub GetFriendList => Vec<UserInfo> {}
);
impl_payload!(
    @[name=get_group_info]
    #[derive(Debug, Clone, PartialEq, Serialize)]
    pub GetGroupInfo => GroupInfo {
        required {
            pub group_id: String,
        }
    }

    #[derive(Debug, Clone, PartialEq, Deserialize)]

    pub GroupInfo{
        pub group_id: String,
        pub group_name: String,
        #[serde(flatten)]
        pub extra:crate::types::Value,
    }
);

impl_payload!(
    @[name=get_group_list]
    #[derive(Debug, Clone, PartialEq, Serialize)]
    pub GetGroupList => Vec<GroupInfo> {}
);
impl_payload!(
    @[name=get_group_member_info]
    #[derive(Debug, Clone, PartialEq, Serialize)]
    pub GetGroupMemberInfo => GroupMemberInfo {
        required {
            pub group_id: String,
            pub user_id: String,
        }
    }

    #[derive(Debug, Clone, PartialEq, Deserialize)]
    pub GroupMemberInfo{
        pub user_id:String,
        pub user_name:String,
        pub user_display_name:String,
    }

);
impl_payload!(
    @[name=get_group_member_list]
    #[derive(Debug, Clone, PartialEq, Serialize)]
    pub GetGroupMemberList => Vec<GroupMemberInfo> {
        required {
            pub group_id: String,
        }
    }
);
impl_payload!(
    @[name=set_group_name]
    #[derive(Debug, Clone, PartialEq, Serialize)]
    pub SetGroupName => () {
        required {
            pub group_id: String,
            pub group_name: String,
        }
    }
);
impl_payload!(
    @[name=leave_group]
    #[derive(Debug, Clone, PartialEq, Serialize)]
    pub LeaveGroup => () {
        required {
            pub group_id: String,
        }
    }
);
impl_payload!(
    @[name=upload_file]
    #[derive(Debug, Clone, PartialEq, Serialize)]
    pub UploadFile => UploadFileResponse {
        required {
            pub file: String,
        }
    }

    #[derive(Debug, Clone, PartialEq, Deserialize)]
    pub UploadFileResponse {
        pub file_id: String,
    }

);
