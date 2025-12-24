use chrono::prelude::*;
use gix::bstr::ByteSlice;

fn main() {
    // 获取 Asia/Shanghai 时区的当前时间
    let shanghai_tz = FixedOffset::east_opt(8 * 3600).unwrap();
    let now = Utc::now().with_timezone(&shanghai_tz);
    let build_time = now.format("%Y-%m-%d %H:%M:%S").to_string();
    
    println!("cargo:rustc-env=BUILD_TIME={}", build_time);

    // 使用 gix 获取 Git 信息
    if let Ok(repo) = gix::discover(".") {
        // 获取 commit hash
        if let Ok(head) = repo.head() {
            if let Some(id) = head.id() {
                let short_hash = id.to_string()[..7].to_string();
                println!("cargo:rustc-env=GIT_HASH={}", short_hash);
            } else {
                println!("cargo:rustc-env=GIT_HASH=unknown");
            }

            // 获取 branch name
            if let Some(name) = head.referent_name() {
                let branch = name.as_bstr().to_string();
                let branch_name = branch.strip_prefix("refs/heads/").unwrap_or(&branch);
                println!("cargo:rustc-env=GIT_BRANCH={}", branch_name);
            } else {
                println!("cargo:rustc-env=GIT_BRANCH=unknown");
            }
        } else {
            println!("cargo:rustc-env=GIT_HASH=unknown");
            println!("cargo:rustc-env=GIT_BRANCH=unknown");
        }

        // 获取 tag (如果当前 commit 有 tag)
        if let Ok(head) = repo.head() {
            if let Some(id) = head.id() {
                let mut tag_name = String::new();
                
                // 遍历所有 tags
                if let Ok(refs) = repo.references() {
                    for reference in refs.all().ok().into_iter().flatten() {
                        if let Ok(r) = reference {
                            if let Some(name) = r.name().as_bstr().to_str().ok() {
                                if name.starts_with("refs/tags/") {
                                    if let Ok(peeled) = r.id().object() {
                                        if peeled.id == id {
                                            tag_name = name.strip_prefix("refs/tags/")
                                                .unwrap_or(name)
                                                .to_string();
                                            break;
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
                
                println!("cargo:rustc-env=GIT_TAG={}", tag_name);
            } else {
                println!("cargo:rustc-env=GIT_TAG=");
            }
        } else {
            println!("cargo:rustc-env=GIT_TAG=");
        }
    } else {
        // 不在 git 仓库中
        println!("cargo:rustc-env=GIT_HASH=unknown");
        println!("cargo:rustc-env=GIT_BRANCH=unknown");
        println!("cargo:rustc-env=GIT_TAG=");
    }

    // 获取目标平台
    let target = std::env::var("TARGET").unwrap_or_else(|_| "unknown".to_string());
    println!("cargo:rustc-env=BUILD_TARGET={}", target);

    // 获取编译模式 (debug/release)
    let profile = std::env::var("PROFILE").unwrap_or_else(|_| "unknown".to_string());
    println!("cargo:rustc-env=BUILD_PROFILE={}", profile);

    // 重新运行条件
    println!("cargo:rerun-if-changed=.git/HEAD");
    println!("cargo:rerun-if-changed=.git/refs");
}
