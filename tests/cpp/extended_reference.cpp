#include <omath/omath.hpp>
#include <omath/engines/source_engine/collider.hpp>
#include <omath/collision/epa_algorithm.hpp>
#include <iomanip>
#include <iostream>
using namespace omath;
struct Rng{std::uint64_t state=0x853c49e6748fea9bULL;float next(float min,float max){state=state*6364136223846793005ULL+1442695040888963407ULL;return min+(max-min)*(static_cast<float>(state>>40)/static_cast<float>(1<<24));}};
void vector(const char*kind,int index,Vector3<float>v){std::cout<<kind<<' '<<index<<' '<<v.x<<' '<<v.y<<' '<<v.z<<'\n';}
int main(){std::cout<<std::setprecision(9);Rng rng;using Ray=collision::Ray<Vector3<float>>;using Tracer=collision::LineTracer<Ray>;
    Triangle<Vector3<float>> triangle{{-1,0,-1},{1,0,-1},{0,0,1}};
    primitives::Aabb<float> aabb{{-1,-1,-1},{1,1,1}};
    primitives::Obb<float> obb{{0,0,0},{0.8f,0.6f,0},{-0.6f,0.8f,0},{0,0,1},{1,2,0.5f}};
    for(int i=0;i<2048;++i){Ray ray{{rng.next(-8,8),rng.next(-8,8),rng.next(-8,8)},{rng.next(-8,8),rng.next(-8,8),rng.next(-8,8)},i%2==0};vector("triangle",i,Tracer::get_ray_hit_point(ray,triangle));vector("aabb",i,Tracer::get_ray_hit_point(ray,aabb));vector("obb",i,Tracer::get_ray_hit_point(ray,obb));}
    std::vector<primitives::Vertex<>> vertices;for(int x:{-1,1})for(int y:{-1,1})for(int z:{-1,1})vertices.push_back({{float(x),float(y),float(z)},{},{}});
    using Collider=source_engine::MeshCollider;using Gjk=collision::GjkAlgorithm<Collider>;using Epa=collision::Epa<Collider>;
    const Collider a{source_engine::Mesh{vertices,{}}};
    for(int i=0;i<128;++i){source_engine::Mesh mesh{vertices,{}};mesh.set_origin({rng.next(-3,3),rng.next(-3,3),rng.next(-3,3)});const Collider b{mesh};const auto hit=Gjk::is_collide_with_simplex_info(a,b);std::cout<<"gjk "<<i<<' '<<hit.hit<<'\n';if(hit.hit){auto epa=Epa::solve(a,b,hit.simplex);std::cout<<"epa "<<i<<' '<<epa.has_value();if(epa)std::cout<<' '<<epa->depth<<' '<<epa->normal.x<<' '<<epa->normal.y<<' '<<epa->normal.z;std::cout<<'\n';}}
    using namespace projectile_prediction;
    ProjPredEngineLegacy<> engine{400.f,0.005f,5.f,5.f};
    for(int i=0;i<64;++i){const Target<float> target{.m_origin={rng.next(50,1000),rng.next(-100,100),rng.next(0,100)},.m_velocity={rng.next(-20,20),rng.next(-20,20),0},.m_is_airborne=false};const Projectile<float> p{.m_origin={3,2,1},.m_launch_speed=1000,.m_gravity_scale=0.4f};const auto point=engine.maybe_calculate_aim_point(p,target);std::cout<<"projectile "<<i<<' '<<point.has_value();if(point)std::cout<<' '<<point->x<<' '<<point->y<<' '<<point->z;std::cout<<'\n';}
}
